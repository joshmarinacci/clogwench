import {
    ActionButton, AssetsDoc,
    BaseParentView,
    BaseView,
    CanvasSurface, COMMAND_ACTION,
    CoolEvent, DebugLayer, Header, LayerView, load_assets_from_json, Point,
    POINTER_DOWN, PointerEvent,
    randi,
    Rect, Sheet,
    Size, Sprite,
    SurfaceContext, VBox
} from "thneed-gfx";

import Socket from "net"

function start() {
    //on init, open a window,
    //fill window with red color

    // let socket = Socket.connect(3333)
    // console.log('created a socket')
    //connect to localhost 3333
    //        match TcpStream::connect("localhost:3333") {
    // send AppConnect
    //    let resp: Result<APICommand, RecvError> = client.send_and_wait(APICommand::AppConnect(HelloApp{}));
    // wait for responseto get the appid
    //        Ok(APICommand::AppConnectResponse(appinfo)) => {
    //             appid = appinfo.app_id
    //         }
    //send open window command
    //    let resp2: Result<APICommand, RecvError> = client.send_and_wait(APICommand::OpenWindowCommand(OpenWindowCommand{
    //         window_type: String::from("plain"),
    //         bounds: bounds,
    //         }));
    //get window response
    //        Ok(APICommand::OpenWindowResponse(wininfo)) => {
    //             winid = wininfo.window_id
    //         }
    // send drawing commands
    // wait for input events
}

