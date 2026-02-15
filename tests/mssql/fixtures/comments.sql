INSERT INTO comment(comment_id, post_id, user_id, content, created_at)
VALUES (1, 1, 2, 'lol bet ur still bad, 1v1 me', DATEADD(MINUTE, -50, SYSUTCDATETIME())),
       (2, 1, 1, 'you''re on!', DATEADD(MINUTE, -45, SYSUTCDATETIME())),
       (3, 2, 1, 'lol you''re just mad you lost :P', DATEADD(MINUTE, -15, SYSUTCDATETIME()));
