"""Read-only verification after the localnet driver has performed shutdown."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import socket

N = Path(__file__).parent
chains = json.loads((N / 'chains.json').read_text())
launchers = json.loads((N / 'launchers.json').read_text())
pids = {row['pid'] for row in launchers}
for row in json.loads((N / 'remaining-node-ownership.json').read_text()):
    pids.update(row['pids'])
pids.add(805591)  # run-live.py launched for this validation run.
remaining = []
for pid in sorted(pids):
    path = Path(f'/proc/{pid}/cmdline')
    if path.exists():
        command = path.read_bytes().replace(b'\0', b' ').decode(errors='replace')
        if str(N) in command:
            remaining.append({'pid': pid, 'command': command})
ports = [chain[base] + offset for chain in chains for base in ['rpc_port', 'p2p_port'] for offset in range(3)] + [9977]
checks = []
for port in ports:
    with socket.socket() as sock:
        checks.append({'port': port, 'closed': sock.connect_ex(('127.0.0.1', port)) != 0})
ancestors = set()
pid = os.getpid()
while pid > 1 and pid not in ancestors:
    ancestors.add(pid)
    status = Path(f"/proc/{pid}/status").read_text()
    pid = int(next(line.split()[1] for line in status.splitlines() if line.startswith("PPid:")))
other_nodes = []
remaining_test_processes = []
for proc in Path('/proc').iterdir():
    if not proc.name.isdigit():
        continue
    try:
        raw_command = (proc / 'cmdline').read_bytes()
        command = raw_command.split(b'\0')
        if int(proc.name) not in ancestors and str(N).encode() in raw_command and command:
            executable = Path(command[0].decode(errors="replace")).name
            if executable.startswith("python") or executable in {"bash", "pytest", "node", "node-subtensor", "uv", "cargo"}:
                remaining_test_processes.append({"pid": int(proc.name), "executable": executable})
        if command and Path(command[0].decode(errors='replace')).name == 'node-subtensor':
            other_nodes.append(int(proc.name))
    except (FileNotFoundError, ProcessLookupError, PermissionError):
        pass
result = {
    'verified_at': datetime.now(timezone.utc).isoformat(),
    'recorded_owned_process_count': len(pids),
    'all_owned_processes_stopped': not remaining,
    'remaining_owned_processes': remaining,
    'all_validation_ports_closed': all(row['closed'] for row in checks),
    'ports': checks,
    'remaining_node_subtensor_pids': other_nodes,
    'remaining_owned_test_processes': remaining_test_processes,
    'scope': 'All recorded launchers/driver and the final three validator PIDs, plus a global process scan for node-subtensor, 90 validation ports and the temporary metadata gate port. Completed launchers removed their PID files during shutdown. No unrelated process was stopped.',
}
(N / 'cleanup.json').write_text(json.dumps(result, indent=2) + '\n')
assert result['all_owned_processes_stopped'] and result['all_validation_ports_closed'], result
assert not other_nodes, f'Other node-subtensor process remains: {other_nodes}'
assert not remaining_test_processes, remaining_test_processes
print(json.dumps({key: result[key] for key in ['all_owned_processes_stopped', 'all_validation_ports_closed', 'remaining_node_subtensor_pids']}))
