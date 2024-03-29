FROM	rust	as	builder
ENV	PKG_CONFIG_ALLOW_CROSS	1
WORKDIR	/usr/src/openlab-rest
COPY	.	.
RUN	cargo install --path .

FROM	debian
COPY	--from=builder /usr/local/cargo/bin/openlab-rest	/usr/local/bin/openlab-rest
CMD	["openlab-rest"]
