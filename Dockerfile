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

FROM --platform=$TARGETPLATFORM alpine:3.17 as runtime

ENV TZ=Asia/Shanghai

RUN sed -i "s/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g" /etc/apk/repositories \
    && apk update  \
    && apk add --no-cache vim tzdata \
    && echo "${TZ}" > /etc/timezone \
    && ln -sf /usr/share/zoneinfo/${TZ} /etc/localtime \
    && rm -rf /var/cache/apk/*

COPY --from=builder /usr/local/cargo/bin/redis-exporter /usr/local/bin/redis-exporter

ENTRYPOINT [ "/usr/local/bin/redis-exporter" ] 

EXPOSE 8090