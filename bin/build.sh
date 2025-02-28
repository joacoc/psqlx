# Apply updates into PSQL
chmod +x ./src/psqlx/bin/updater.sh
src/psqlx/bin/updater.sh

# Configure PSQL
cd external/psql
./configure --with-openssl
make clean
make -C src/interfaces/libpq
/sudo make install -C src/interfaces/libpq
make -C src/common
sudo make install -C src/common
sudo make install -C src/port

cd ../..

cargo build --release
make -C external/psql/src/bin/psql
sudo make install -C external/psql/src/bin/psql

cp target/release/libpsqlx_ai.so ~/.local/share/psqlx/plugins/

/usr/local/pgsql/bin/psqlx "postgresql://BenchmarkRole:8MGUYwHcu6Da@ep-lively-tooth-a49kswzq-pooler.us-east-1.aws.neon.tech/neondb?sslmode=require"