# Apply updates into PSQL
chmod +x ./src/psqlx/bin/updater.sh
src/psqlx/bin/updater.sh

# Configure PSQL
cd external/psql
./configure --with-openssl
make clean
make -C src/interfaces/libpq
make install -C src/interfaces/libpq
make -C src/common
make install -C src/common
make install -C src/port

cd ../..

cargo build --release
make -C external/psql/src/bin/psql
make install -C external/psql/src/bin/psql