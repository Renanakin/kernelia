import { json } from '@sveltejs/kit';
import os from 'node:os';
import { exec } from 'node:child_process';
import { promisify } from 'node:util';

const execAsync = promisify(exec);

export async function GET() {
  try {
    const interfaces = os.networkInterfaces();
    let physicalIp = null;
    let fallbackIp = null;
    const adapterDetails = [];

    for (const name of Object.keys(interfaces)) {
      const isVirtual = /vEthernet|WSL|Docker|VirtualBox|VMware|VPN|Loopback/i.test(name);
      for (const net of interfaces[name] || []) {
        if ((net.family === 'IPv4' || net.family === 4) && !net.internal) {
          adapterDetails.push({
            name,
            ip: net.address,
            netmask: net.netmask,
            is_virtual: isVirtual,
          });
          if (!isVirtual && !physicalIp) {
            physicalIp = net.address;
          } else if (!fallbackIp) {
            fallbackIp = net.address;
          }
        }
      }
    }

    const localIp = physicalIp || fallbackIp || '127.0.0.1';

    let disks = [];
    try {
      const { stdout } = await execAsync(
        'powershell -NoProfile -Command "Get-CimInstance Win32_LogicalDisk | Select-Object DeviceID, FreeSpace, Size, VolumeName | ConvertTo-Json"'
      );
      const parsed = JSON.parse(stdout);
      const list = Array.isArray(parsed) ? parsed : [parsed];
      disks = list
        .filter((d) => d && d.DeviceID)
        .map((d) => {
          const totalGb = d.Size ? (d.Size / (1024 ** 3)).toFixed(1) + ' GB' : 'N/A';
          const freeGb = d.FreeSpace ? (d.FreeSpace / (1024 ** 3)).toFixed(1) + ' GB' : 'N/A';
          return {
            name: d.DeviceID,
            mount_point: d.DeviceID,
            total_space: totalGb,
            available_space: freeGb,
            volume_name: d.VolumeName || 'Disco Local',
          };
        });
    } catch {
      disks = [
        { name: 'C:', mount_point: 'C:', total_space: 'Unidad Principal', available_space: 'Ok' },
      ];
    }

    const totalMemGb = (os.totalmem() / (1024 ** 3)).toFixed(1) + ' GB';
    const freeMemGb = (os.freemem() / (1024 ** 3)).toFixed(1) + ' GB';
    const usedMemGb = ((os.totalmem() - os.freemem()) / (1024 ** 3)).toFixed(1) + ' GB';

    return json({
      hostname: os.hostname(),
      platform: os.platform(),
      arch: os.arch(),
      local_ip: localIp,
      adapters: adapterDetails,
      cpu_count: os.cpus().length,
      cpu_model: os.cpus()[0]?.model || 'Generic CPU',
      cpu_usage: `${Math.round((1 - os.freemem() / os.totalmem()) * 100)}%`,
      memory_used: usedMemGb,
      memory_total: totalMemGb,
      memory_free: freeMemGb,
      disks: disks,
      source: 'node-bridge-pc',
    });
  } catch (error) {
    return json({ error: String(error?.message || error) }, { status: 500 });
  }
}
