// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use jsonrpsee::core::RpcResult;

/// Wait for executor capacity instead of presenting ordinary backpressure as an
/// EVM failure. Gas estimation shares one deadline across all its probes. Every
/// retry still passes executor admission, and cancellation drops the sleep
/// without launching another probe.
pub async fn retry_execution_capacity<T, F: std::future::Future<Output = RpcResult<T>>>(
	mut execute: impl FnMut() -> F,
	deadline: tokio::time::Instant,
) -> RpcResult<T> {
	loop {
		let result = execute().await;
		let retry = result.as_ref().err().is_some_and(|error| {
			error.code() == jsonrpsee::types::error::INTERNAL_ERROR_CODE
				&& error.message()
					== format!(
						"runtime error: Execution failed: Other: {}",
						fp_rpc::LOCAL_EXECUTION_CAPACITY_ERROR
					)
		});
		let now = tokio::time::Instant::now();
		if !retry || now >= deadline {
			return result;
		}
		tokio::time::sleep_until(deadline.min(now + std::time::Duration::from_millis(125))).await;
	}
}
