FROM postgres:latest

# Files copied to this subdirectory on the container will run in
# alphabetical order
COPY *.sql /docker-entrypoint-initdb.d/

