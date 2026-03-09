import pc from 'picocolors';

export function printJson(data: unknown, format: string): void {
  if (format === 'json') {
    console.log(JSON.stringify(data));
  } else {
    console.log(JSON.stringify(data, null, 2));
  }
}

export function success(msg: string): void {
  console.error(pc.green('✓') + ' ' + msg);
}

export function error(msg: string): void {
  console.error(pc.red('error') + ': ' + msg);
}
