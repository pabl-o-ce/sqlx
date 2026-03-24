CREATE TABLE post (
    post_id    INT NOT NULL IDENTITY(1,1) PRIMARY KEY,
    user_id    INT NOT NULL REFERENCES [user](user_id),
    content    NVARCHAR(MAX) NOT NULL,
    created_at DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME()
);
CREATE INDEX post_created_at ON post (created_at DESC);
