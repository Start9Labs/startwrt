import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';


 // @ts-ignore
import { network } from './libluci/network';
// @ts-ignore
window['network'] = network;

bootstrapApplication(AppComponent, appConfig)
  .catch((err) => console.error(err));

