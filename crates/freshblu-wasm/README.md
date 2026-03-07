# freshblu-wasm

WASM client library for FreshBlu — works in browsers and Node.js.

## Building

```bash
# For browsers
wasm-pack build --target web

# For Node.js
wasm-pack build --target nodejs

# For bundlers (Webpack/Vite)
wasm-pack build --target bundler
```

## Browser Usage

```html
<script type="module">
import init, { FreshBluConfig, FreshBluHttp } from './pkg/freshblu_wasm.js';

await init();

const config = new FreshBluConfig('localhost', 3000);
const client = new FreshBluHttp(config);

// Check server status
const status = await client.status();
console.log(status);

// Register a device
const device = await client.register('sensor');
console.log(device.uuid, device.token);
</script>
```

## Node.js Usage

```javascript
const { FreshBluConfig, FreshBluHttp } = require('./pkg/freshblu_wasm.js');

const config = new FreshBluConfig('localhost', 3000);
const client = new FreshBluHttp(config);
const status = await client.status();
```

## Testing

```bash
wasm-pack test --node
wasm-pack test --headless --chrome
```

## License

MIT OR Apache-2.0
