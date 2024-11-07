import { inject, Injectable } from '@angular/core';
import { BehaviorSubject, map, Observable } from 'rxjs';

import { rpc } from '../libluci/rpc';

const callLogin = rpc.declare({
  object: 'session',
  method: 'login',
  params: ['username', 'password', 'timeout'],
  reject: true,
})

export async function rpcLogin() {
  let login_response = await callLogin('root', '', 0);
  rpc.setSessionID(login_response.ubus_rpc_session);
}

@Injectable({
  providedIn: 'root',
})
export class NetworkService {

}
