import {ConstantBackoff, LinearBackoff, WebsocketBuilder} from 'websocket-ts';

console.log("reloadedz")

const ws = new WebsocketBuilder('ws://127.0.0.1:63086/ws')
    .withBackoff(new ConstantBackoff(3000))
    .onOpen((i, ev) => console.log(ev))
    .build();