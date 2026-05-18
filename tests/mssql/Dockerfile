ARG VERSION
FROM mcr.microsoft.com/mssql/server:${VERSION}

# Build steps need root: recent mssql/server images default to USER mssql (10001).
USER root

# Create a config directory
RUN mkdir -p /usr/config
WORKDIR /usr/config

# Bundle config source
COPY mssql/entrypoint.sh /usr/config/entrypoint.sh
COPY mssql/configure-db.sh /usr/config/configure-db.sh
COPY mssql/setup.sql /usr/config/setup.sql

# Grant permissions for to our scripts to be executable
RUN chmod +x /usr/config/entrypoint.sh
RUN chmod +x /usr/config/configure-db.sh
RUN chown 10001 /usr/config/entrypoint.sh
RUN chown 10001 /usr/config/configure-db.sh
USER 10001

ENTRYPOINT ["/usr/config/entrypoint.sh"]
