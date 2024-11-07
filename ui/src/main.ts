import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';

import { Rpc, rpc } from './libluci/rpc';
import { Network, network } from './libluci/network';
import { rpcLogin } from './services/network.service';

declare global {
  var luci: { rpc: Rpc, network: Network };
}
globalThis.luci = { rpc: rpc, network: network };
rpcLogin();

bootstrapApplication(AppComponent, appConfig)
  .catch((err) => console.error(err));

