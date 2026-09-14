-- One Kafka/DB job can encode the full ABR ladder (one source decode).
ALTER TABLE ims_encode_batches
    ADD COLUMN pack_ladder BOOLEAN NOT NULL DEFAULT FALSE;
