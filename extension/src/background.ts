import {ConstantBackoff, WebsocketBuilder} from 'websocket-ts';

new WebsocketBuilder('ws://127.0.0.1:63086/ws')
    .withBackoff(new ConstantBackoff(3000))
    .onOpen((_, ev) => console.log(`Connected!`))
    .onMessage((_, ev) => processMessage(JSON.parse(ev.data as string)))
    .onClose((_, ev) => console.log(`Disconnected!`))
    .build();

type Message = {
    Multiple : Message[],
} | {
    UpdateTimer: Timer | null
} | {
    UpdateTimerState : TimerState
};

type Timer = {
    profile: {
        blocking: {
            websites: string[],
            hide_web_video: boolean,
        }
    },

    state: TimerState,
};

type TimerState = {

}

let processMessage = (msg: Message) => {
    if ("Multiple" in msg) {
        msg.Multiple.forEach(processMessage);
    } else if ("UpdateTimer" in msg) {

    } else if ("UpdateTimerState" in msg) {

    }
}