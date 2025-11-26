# @polkadot-api/logs-provider

This package exports a `JsonRpcProvider` enhancer to record logs of messages sent & received, and a provider that reads from these logs to replay existing sessions.

This can be useful to set up reproduction cases with a provider that behaves consistently.

## withLogsRecorder

```ts
function withLogsRecorder(
  persistLog: (line: string) => void,
  provider: JsonRpcProvider,
): JsonRpcProvider
```

## logsProvider

```ts
function logsProvider(logs: string[]): JsonRpcProvider
```
