IF DB_ID('sqlx') IS NULL
    BEGIN
        CREATE DATABASE sqlx;
    END;
GO

USE sqlx;
GO

IF OBJECT_ID('tweet') IS NULL
    BEGIN
        CREATE TABLE tweet
        (
            id       BIGINT          NOT NULL PRIMARY KEY,
            text     NVARCHAR(4000)  NOT NULL,
            is_sent  TINYINT         NOT NULL DEFAULT 1,
            owner_id BIGINT
        );
    END;
GO

IF OBJECT_ID('tweet_reply') IS NULL
    BEGIN
        CREATE TABLE tweet_reply
        (
            id       BIGINT          NOT NULL IDENTITY(1,1) PRIMARY KEY,
            tweet_id BIGINT          NOT NULL,
            text     NVARCHAR(4000)  NOT NULL,
            owner_id BIGINT,
            CONSTRAINT tweet_id_fk FOREIGN KEY (tweet_id) REFERENCES tweet(id)
        );
    END;
GO

IF OBJECT_ID('products') IS NULL
    BEGIN
        CREATE TABLE products
        (
            product_no INT,
            name       NVARCHAR(200),
            price      DECIMAL(10,2),
            CONSTRAINT chk_price CHECK (price > 0)
        );
    END;
GO
