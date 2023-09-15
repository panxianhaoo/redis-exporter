FROM rust:1.72 as builder 

WORKDIR /usr/src

COPY . .

RUN  echo "[source.crates-io]\n\ 
replace-with = 'rsproxy-sparse'\n\
[source.rsproxy]\n\
registry = \"https://rsproxy.cn/crates.io-index\"\n\
[source.rsproxy-sparse]\n\ 
registry = \"sparse+https://rsproxy.cn/index/\"\n\ 
[registries.rsproxy]\n\ 
index = \"https://rsproxy.cn/crates.io-index\"\n\ 
[net]\n\
git-fetch-with-cli = true\n" >> $CARGO_HOME/config

RUN cargo install --path . --verbose

FROM debian

RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*

ENV TZ=Asia/Shanghai


COPY --from=builder /usr/local/cargo/bin/redis-exporter /redis-exporter

USER 1000

RUN pwd

ENTRYPOINT [ "./redis-exporter" ] 

EXPOSE 8090