const LOG_INDENT = "      ";

export const log = {
  tx: (label: string, msg: string) => console.log(`${LOG_INDENT}[${label}] ${msg}`),
  info: (msg: string) => console.log(`${LOG_INDENT}${msg}`),
  error: (label: string, msg: string) => console.error(`${LOG_INDENT}[${label}] ${msg}`),
};
