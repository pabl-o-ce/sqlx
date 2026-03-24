SET IDENTITY_INSERT post ON;
INSERT INTO post(post_id, user_id, content, created_at)
VALUES (1, 1, 'This new computer is lightning-fast!', DATEADD(HOUR, -1, SYSUTCDATETIME())),
       (2, 2, '@alice is a haxxor :(', DATEADD(MINUTE, -30, SYSUTCDATETIME()));
SET IDENTITY_INSERT post OFF;
