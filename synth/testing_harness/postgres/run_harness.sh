PORT=5433
PASSWORD="mysecretpassword"
NAME=postgres-synth-harness

docker build -t $NAME . || exit 1;
docker run -p $PORT:5432 -e POSTGRES_PASSWORD=$PASSWORD -d $NAME;
echo "Running database with container name $NAME on port $PORT with password $PASSWORD"
