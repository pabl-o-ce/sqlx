SET IDENTITY_INSERT [user] ON;
INSERT INTO [user](user_id, username) VALUES (1, 'alice'), (2, 'bob');
SET IDENTITY_INSERT [user] OFF;
