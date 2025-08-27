-- OCPP Database Initialization Script
-- This script creates the necessary tables and indexes for OCPP 1.6J implementation

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create custom types
CREATE TYPE charge_point_status AS ENUM (
    'Available',
    'Preparing',
    'Charging',
    'SuspendedEV',
    'SuspendedEVSE',
    'Finishing',
    'Reserved',
    'Faulted',
    'Unavailable'
);

CREATE TYPE authorization_status AS ENUM (
    'Accepted',
    'Blocked',
    'Expired',
    'Invalid',
    'ConcurrentTx'
);

CREATE TYPE transaction_stop_reason AS ENUM (
    'EmergencyStop',
    'EVDisconnected',
    'HardReset',
    'Local',
    'Other',
    'PowerLoss',
    'Reboot',
    'Remote',
    'SoftReset',
    'UnlockCommand',
    'DeAuthorized'
);

CREATE TYPE registration_status AS ENUM (
    'Accepted',
    'Pending',
    'Rejected'
);

CREATE TYPE error_code AS ENUM (
    'ConnectorLockFailure',
    'EVCommunicationError',
    'GroundFailure',
    'HighTemperature',
    'InternalError',
    'LocalListConflict',
    'NoError',
    'OtherError',
    'OverCurrentFailure',
    'OverVoltage',
    'PowerMeterFailure',
    'PowerSwitchFailure',
    'ReaderFailure',
    'ResetFailure',
    'UnderVoltage',
    'WeakSignal'
);

-- Charge points table
CREATE TABLE charge_points (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id VARCHAR(255) UNIQUE NOT NULL,
    vendor VARCHAR(20) NOT NULL,
    model VARCHAR(20) NOT NULL,
    serial_number VARCHAR(25),
    firmware_version VARCHAR(50),
    iccid VARCHAR(20),
    imsi VARCHAR(20),
    meter_type VARCHAR(25),
    meter_serial_number VARCHAR(25),
    endpoint_url TEXT,
    registration_status registration_status DEFAULT 'Pending',
    last_boot_notification TIMESTAMPTZ,
    last_heartbeat TIMESTAMPTZ,
    heartbeat_interval INTEGER DEFAULT 60,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create index on charge_point_id for fast lookups
CREATE INDEX idx_charge_points_id ON charge_points(charge_point_id);
CREATE INDEX idx_charge_points_status ON charge_points(registration_status);
CREATE INDEX idx_charge_points_last_heartbeat ON charge_points(last_heartbeat);

-- Connectors table
CREATE TABLE connectors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    connector_id INTEGER NOT NULL,
    status charge_point_status DEFAULT 'Available',
    error_code error_code DEFAULT 'NoError',
    info TEXT,
    vendor_id VARCHAR(255),
    vendor_error_code VARCHAR(50),
    status_timestamp TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(charge_point_id, connector_id)
);

-- Create indexes on connectors
CREATE INDEX idx_connectors_charge_point ON connectors(charge_point_id);
CREATE INDEX idx_connectors_status ON connectors(status);
CREATE INDEX idx_connectors_composite ON connectors(charge_point_id, connector_id);

-- ID tags (RFID cards, etc.) table
CREATE TABLE id_tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    id_tag VARCHAR(20) UNIQUE NOT NULL,
    parent_id_tag VARCHAR(20),
    expiry_date TIMESTAMPTZ,
    status authorization_status DEFAULT 'Accepted',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create index on id_tag for fast lookups
CREATE INDEX idx_id_tags_tag ON id_tags(id_tag);
CREATE INDEX idx_id_tags_status ON id_tags(status);
CREATE INDEX idx_id_tags_parent ON id_tags(parent_id_tag);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id INTEGER UNIQUE NOT NULL,
    charge_point_id UUID NOT NULL REFERENCES charge_points(id),
    connector_id INTEGER NOT NULL,
    id_tag VARCHAR(20) NOT NULL,
    start_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    meter_start INTEGER NOT NULL DEFAULT 0,
    reservation_id INTEGER,
    stop_timestamp TIMESTAMPTZ,
    meter_stop INTEGER,
    stop_id_tag VARCHAR(20),
    stop_reason transaction_stop_reason,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (charge_point_id, connector_id) REFERENCES connectors(charge_point_id, connector_id)
);

-- Create indexes on transactions
CREATE INDEX idx_transactions_id ON transactions(transaction_id);
CREATE INDEX idx_transactions_charge_point ON transactions(charge_point_id);
CREATE INDEX idx_transactions_connector ON transactions(charge_point_id, connector_id);
CREATE INDEX idx_transactions_id_tag ON transactions(id_tag);
CREATE INDEX idx_transactions_start_time ON transactions(start_timestamp);
CREATE INDEX idx_transactions_active ON transactions(stop_timestamp) WHERE stop_timestamp IS NULL;

-- Meter values table
CREATE TABLE meter_values (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id INTEGER REFERENCES transactions(transaction_id),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id),
    connector_id INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sampled_values JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (charge_point_id, connector_id) REFERENCES connectors(charge_point_id, connector_id)
);

-- Create indexes on meter values
CREATE INDEX idx_meter_values_transaction ON meter_values(transaction_id);
CREATE INDEX idx_meter_values_charge_point ON meter_values(charge_point_id);
CREATE INDEX idx_meter_values_timestamp ON meter_values(timestamp);
CREATE INDEX idx_meter_values_sampled_gin ON meter_values USING GIN(sampled_values);

-- Configuration keys table
CREATE TABLE configuration_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    key VARCHAR(50) NOT NULL,
    value TEXT,
    readonly BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(charge_point_id, key)
);

-- Create indexes on configuration keys
CREATE INDEX idx_config_keys_charge_point ON configuration_keys(charge_point_id);
CREATE INDEX idx_config_keys_key ON configuration_keys(key);

-- Reservations table
CREATE TABLE reservations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reservation_id INTEGER UNIQUE NOT NULL,
    charge_point_id UUID NOT NULL REFERENCES charge_points(id),
    connector_id INTEGER NOT NULL,
    id_tag VARCHAR(20) NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL,
    parent_id_tag VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (charge_point_id, connector_id) REFERENCES connectors(charge_point_id, connector_id)
);

-- Create indexes on reservations
CREATE INDEX idx_reservations_id ON reservations(reservation_id);
CREATE INDEX idx_reservations_charge_point ON reservations(charge_point_id);
CREATE INDEX idx_reservations_connector ON reservations(charge_point_id, connector_id);
CREATE INDEX idx_reservations_id_tag ON reservations(id_tag);
CREATE INDEX idx_reservations_expiry ON reservations(expiry_date);

-- Charging profiles table (for Smart Charging feature)
CREATE TABLE charging_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charging_profile_id INTEGER NOT NULL,
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    transaction_id INTEGER REFERENCES transactions(transaction_id),
    stack_level INTEGER NOT NULL,
    charging_profile_purpose VARCHAR(50) NOT NULL,
    charging_profile_kind VARCHAR(50) NOT NULL,
    recurrency_kind VARCHAR(20),
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    charging_schedule JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(charge_point_id, charging_profile_id)
);

-- Create indexes on charging profiles
CREATE INDEX idx_charging_profiles_charge_point ON charging_profiles(charge_point_id);
CREATE INDEX idx_charging_profiles_transaction ON charging_profiles(transaction_id);
CREATE INDEX idx_charging_profiles_stack_level ON charging_profiles(charge_point_id, stack_level);
CREATE INDEX idx_charging_profiles_validity ON charging_profiles(valid_from, valid_to);

-- Local auth list table
CREATE TABLE local_auth_list (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    id_tag VARCHAR(20) NOT NULL,
    id_tag_info JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(charge_point_id, id_tag)
);

-- Create indexes on local auth list
CREATE INDEX idx_local_auth_charge_point ON local_auth_list(charge_point_id);
CREATE INDEX idx_local_auth_id_tag ON local_auth_list(id_tag);

-- Firmware updates table
CREATE TABLE firmware_updates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    location TEXT NOT NULL,
    retrieve_date TIMESTAMPTZ NOT NULL,
    retries INTEGER DEFAULT 3,
    retry_interval INTEGER DEFAULT 60,
    status VARCHAR(50) DEFAULT 'Idle',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes on firmware updates
CREATE INDEX idx_firmware_updates_charge_point ON firmware_updates(charge_point_id);
CREATE INDEX idx_firmware_updates_status ON firmware_updates(status);

-- Diagnostics table
CREATE TABLE diagnostics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    location TEXT NOT NULL,
    retries INTEGER DEFAULT 3,
    retry_interval INTEGER DEFAULT 60,
    start_time TIMESTAMPTZ,
    stop_time TIMESTAMPTZ,
    file_name VARCHAR(255),
    status VARCHAR(50) DEFAULT 'Idle',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes on diagnostics
CREATE INDEX idx_diagnostics_charge_point ON diagnostics(charge_point_id);
CREATE INDEX idx_diagnostics_status ON diagnostics(status);

-- Message log table for debugging and auditing
CREATE TABLE message_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID REFERENCES charge_points(id),
    message_type VARCHAR(20) NOT NULL,
    direction VARCHAR(10) NOT NULL, -- 'inbound' or 'outbound'
    unique_id VARCHAR(255),
    action VARCHAR(50),
    payload JSONB,
    raw_message TEXT,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    processing_time_ms INTEGER,
    error_message TEXT
);

-- Create indexes on message log
CREATE INDEX idx_message_log_charge_point ON message_log(charge_point_id);
CREATE INDEX idx_message_log_timestamp ON message_log(timestamp);
CREATE INDEX idx_message_log_type_direction ON message_log(message_type, direction);
CREATE INDEX idx_message_log_action ON message_log(action);
CREATE INDEX idx_message_log_unique_id ON message_log(unique_id);

-- Sessions table for WebSocket connection tracking
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    charge_point_id UUID NOT NULL REFERENCES charge_points(id) ON DELETE CASCADE,
    session_id VARCHAR(255) UNIQUE NOT NULL,
    websocket_protocol VARCHAR(50),
    remote_address INET,
    connected_at TIMESTAMPTZ DEFAULT NOW(),
    disconnected_at TIMESTAMPTZ,
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(charge_point_id, session_id)
);

-- Create indexes on sessions
CREATE INDEX idx_sessions_charge_point ON sessions(charge_point_id);
CREATE INDEX idx_sessions_active ON sessions(disconnected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_sessions_last_seen ON sessions(last_seen);

-- Create trigger functions for updating timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for all tables with updated_at columns
CREATE TRIGGER update_charge_points_updated_at BEFORE UPDATE ON charge_points FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_connectors_updated_at BEFORE UPDATE ON connectors FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_id_tags_updated_at BEFORE UPDATE ON id_tags FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_configuration_keys_updated_at BEFORE UPDATE ON configuration_keys FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_reservations_updated_at BEFORE UPDATE ON reservations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_charging_profiles_updated_at BEFORE UPDATE ON charging_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_local_auth_list_updated_at BEFORE UPDATE ON local_auth_list FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_firmware_updates_updated_at BEFORE UPDATE ON firmware_updates FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_diagnostics_updated_at BEFORE UPDATE ON diagnostics FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create auto-increment sequence for transaction IDs
CREATE SEQUENCE transaction_id_seq START 1;
ALTER TABLE transactions ALTER COLUMN transaction_id SET DEFAULT nextval('transaction_id_seq');

-- Create auto-increment sequence for reservation IDs
CREATE SEQUENCE reservation_id_seq START 1;
ALTER TABLE reservations ALTER COLUMN reservation_id SET DEFAULT nextval('reservation_id_seq');

-- Insert some default configuration keys for testing
INSERT INTO charge_points (charge_point_id, vendor, model, registration_status)
VALUES ('CP001', 'TestVendor', 'TestModel', 'Accepted')
ON CONFLICT (charge_point_id) DO NOTHING;

-- Get the charge point UUID for the test charge point
DO $$
DECLARE
    cp_uuid UUID;
BEGIN
    SELECT id INTO cp_uuid FROM charge_points WHERE charge_point_id = 'CP001';

    -- Insert default connectors
    INSERT INTO connectors (charge_point_id, connector_id, status) VALUES
    (cp_uuid, 1, 'Available'),
    (cp_uuid, 2, 'Available')
    ON CONFLICT (charge_point_id, connector_id) DO NOTHING;

    -- Insert default configuration keys
    INSERT INTO configuration_keys (charge_point_id, key, value, readonly) VALUES
    (cp_uuid, 'HeartbeatInterval', '60', false),
    (cp_uuid, 'MeterValueSampleInterval', '300', false),
    (cp_uuid, 'NumberOfConnectors', '2', true),
    (cp_uuid, 'SupportedFeatureProfiles', 'Core,FirmwareManagement,LocalAuthListManagement,Reservation,SmartCharging,RemoteTrigger', true),
    (cp_uuid, 'AuthorizeRemoteTxRequests', 'true', false),
    (cp_uuid, 'LocalAuthorizeOffline', 'true', false),
    (cp_uuid, 'LocalPreAuthorize', 'false', false),
    (cp_uuid, 'AllowOfflineTxForUnknownId', 'false', false),
    (cp_uuid, 'TransactionMessageAttempts', '3', false),
    (cp_uuid, 'TransactionMessageRetryInterval', '60', false),
    (cp_uuid, 'StopTransactionOnInvalidId', 'true', false),
    (cp_uuid, 'StopTransactionOnEVSideDisconnect', 'true', false),
    (cp_uuid, 'MeterValuesAlignedData', 'Energy.Active.Import.Register', false),
    (cp_uuid, 'MeterValuesSampledData', 'Energy.Active.Import.Register', false),
    (cp_uuid, 'ClockAlignedDataInterval', '0', false),
    (cp_uuid, 'ConnectionTimeOut', '60', false),
    (cp_uuid, 'WebSocketPingInterval', '300', false)
    ON CONFLICT (charge_point_id, key) DO NOTHING;
END $$;

-- Insert some test ID tags
INSERT INTO id_tags (id_tag, status) VALUES
('TAG001', 'Accepted'),
('TAG002', 'Accepted'),
('BLOCKED001', 'Blocked'),
('EXPIRED001', 'Expired')
ON CONFLICT (id_tag) DO NOTHING;

-- Create views for common queries

-- Active transactions view
CREATE OR REPLACE VIEW active_transactions AS
SELECT
    t.*,
    cp.charge_point_id,
    cp.vendor,
    cp.model
FROM transactions t
JOIN charge_points cp ON t.charge_point_id = cp.id
WHERE t.stop_timestamp IS NULL;

-- Connector status view
CREATE OR REPLACE VIEW connector_status AS
SELECT
    cp.charge_point_id,
    c.connector_id,
    c.status,
    c.error_code,
    c.info,
    c.status_timestamp,
    t.transaction_id,
    t.id_tag as current_id_tag
FROM connectors c
JOIN charge_points cp ON c.charge_point_id = cp.id
LEFT JOIN active_transactions t ON c.charge_point_id = t.charge_point_id AND c.connector_id = t.connector_id;

-- Charge point overview
CREATE OR REPLACE VIEW charge_point_overview AS
SELECT
    cp.charge_point_id,
    cp.vendor,
    cp.model,
    cp.registration_status,
    cp.last_heartbeat,
    cp.heartbeat_interval,
    COUNT(c.id) as connector_count,
    COUNT(at.id) as active_transactions,
    s.connected_at,
    s.last_seen
FROM charge_points cp
LEFT JOIN connectors c ON cp.id = c.charge_point_id
LEFT JOIN active_transactions at ON cp.id = at.charge_point_id
LEFT JOIN sessions s ON cp.id = s.charge_point_id AND s.disconnected_at IS NULL
GROUP BY cp.id, s.connected_at, s.last_seen;

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO ocpp_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO ocpp_user;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO ocpp_user;

-- Create database user roles for different access levels
DO $$
BEGIN
    -- Read-only role for reporting
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'ocpp_readonly') THEN
        CREATE ROLE ocpp_readonly;
        GRANT CONNECT ON DATABASE ocpp_dev TO ocpp_readonly;
        GRANT USAGE ON SCHEMA public TO ocpp_readonly;
        GRANT SELECT ON ALL TABLES IN SCHEMA public TO ocpp_readonly;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO ocpp_readonly;
    END IF;

    -- Application role with full access
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'ocpp_app') THEN
        CREATE ROLE ocpp_app;
        GRANT CONNECT ON DATABASE ocpp_dev TO ocpp_app;
        GRANT USAGE ON SCHEMA public TO ocpp_app;
        GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO ocpp_app;
        GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO ocpp_app;
        GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO ocpp_app;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO ocpp_app;
        ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO ocpp_app;
    END IF;
EXCEPTION
    WHEN duplicate_object THEN
        NULL; -- Role already exists, ignore
END $$;

-- Ensure the main user has all necessary permissions
GRANT ocpp_app TO ocpp_user;

COMMIT;
