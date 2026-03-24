CREATE TABLE comment (
    comment_id INT NOT NULL PRIMARY KEY,
    post_id    INT NOT NULL REFERENCES post(post_id),
    user_id    INT NOT NULL REFERENCES [user](user_id),
    content    NVARCHAR(MAX) NOT NULL,
    created_at DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME()
);
CREATE INDEX comment_created_at ON comment (created_at DESC);
