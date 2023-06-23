# Backward Compatibility -  Example

This example shows how to create an opentelemetry pipeline to send traces to the receiver that requires TraceID to be in 64-bit format.
Setting the **with_backward_compatible** flag in the Trace config will generate a 128-bit TraceID with high 64-bits set to Zeros. For example, `0000000000000000830f23b7f0bf5f84`.
The receiver that requires 64-bit TraceID will only pick the low 64-bits by default.

The default value of **with_backward_compatible** flag is false and not setting this or setting it to false will generate 128-bit TraceID. For example, `62a7aade3f9ce7174c6b2c859398d856`.

## Usage

First make sure you have a running version of the opentelemetry collector you want to send data to:

```shell
$ docker run -p 4317:4317 otel/opentelemetry-collector-dev:latest
```

```shell
$ cargo run
```



