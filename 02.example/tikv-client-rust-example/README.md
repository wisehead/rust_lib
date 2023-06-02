# Use tikv client with rust to test.

### Deploy a playground with binary

TiKV is able to run separatedly with PD, which is the minimal deployment required.

1. Download and extract binaries.

   ```
   $ export TIKV_VERSION=v4.0.12
   $ export GOOS=darwin  # only {darwin, linux} are supported
   $ export GOARCH=amd64 # only {amd64, arm64} are supported
   $ curl -O  https://tiup-mirrors.pingcap.com/tikv-$TIKV_VERSION-$GOOS-$GOARCH.tar.gz
   $ curl -O  https://tiup-mirrors.pingcap.com/pd-$TIKV_VERSION-$GOOS-$GOARCH.tar.gz
   $ tar -xzf tikv-$TIKV_VERSION-$GOOS-$GOARCH.tar.gz
   $ tar -xzf pd-$TIKV_VERSION-$GOOS-$GOARCH.tar.gz
   ```

2. Start PD instance.

   ```
   $ ./pd-server --name=pd --data-dir=/tmp/pd/data --client-urls="http://127.0.0.1:2379" --peer-urls="http://127.0.0.1:2380" --initial-cluster="pd=http://127.0.0.1:2380" --log-file=/tmp/pd/log/pd.log
   ```

3. Start TiKV instance.

   ```
   $ ./tikv-server --pd-endpoints="127.0.0.1:2379" --addr="127.0.0.1:20160" --data-dir=/tmp/tikv/data --log-file=/tmp/tikv/log/tikv.log
   ```

4. use tikv clien to test. 