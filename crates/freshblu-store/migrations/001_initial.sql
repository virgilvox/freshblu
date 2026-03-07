CREATE TABLE IF NOT EXISTS devices (
    uuid        UUID PRIMARY KEY,
    data        JSONB NOT NULL,
    online      BOOLEAN DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_devices_type ON devices ((data->>'type'));
CREATE INDEX IF NOT EXISTS idx_devices_online ON devices (online) WHERE online = true;

CREATE TABLE IF NOT EXISTS tokens (
    id          BIGSERIAL PRIMARY KEY,
    device_uuid UUID NOT NULL REFERENCES devices(uuid) ON DELETE CASCADE,
    hash        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_on  BIGINT,
    tag         TEXT
);
CREATE INDEX IF NOT EXISTS idx_tokens_device ON tokens(device_uuid);

CREATE TABLE IF NOT EXISTS subscriptions (
    emitter_uuid      UUID NOT NULL REFERENCES devices(uuid) ON DELETE CASCADE,
    subscriber_uuid   UUID NOT NULL REFERENCES devices(uuid) ON DELETE CASCADE,
    subscription_type TEXT NOT NULL,
    PRIMARY KEY (emitter_uuid, subscriber_uuid, subscription_type)
);
CREATE INDEX IF NOT EXISTS idx_subs_emitter ON subscriptions(emitter_uuid, subscription_type);
CREATE INDEX IF NOT EXISTS idx_subs_subscriber ON subscriptions(subscriber_uuid);
