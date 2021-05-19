#[allow(dead_code)]
/// Messages that the server can send to the client
pub enum ServerMessage {
    Welcome = 1,
    Kicked = 2,
    ClientDisconnect = 3,
    RoomPlayerAdd = 4,
    RoomPlayerUpdatedReadyState = 5,
    FailCreateGame = 6,
    ErrorJoinGame = 7,
    ClientInfo = 8,
    GameInfo = 9,
    UpdatedPigIcon = 10,
    SettingsValueChanged = 11,
    PigItemValueChanged = 12,
    PigConfigValueChanged = 13,
    RoomTimerUpdate = 14,
    BothClientsLoadedGame = 15,
    GameTimerUpdate = 16,
    GamePlayerUpdatedReadyState = 17,
    OpponentPigPlacement = 18,
    MoveData = 19,
    TurnInit = 20,
    TurnSecondUpdate = 21,
    Win = 22,
    EnemyPieceData = 23,
    ClientPlayAgain = 24,
}

#[allow(dead_code)]
/// Messages the client can send to the server
pub enum ClientMessage {
    GameRequestSent = 1,
    UpdateReadyState = 2,
    UpdatePigIcon = 3,
    UpdateSettingsValue = 4,
    UpdatePigItemValue = 5,
    FinishedSceneLoad = 6,
    GamePlayerReadyData = 7,
    Move = 8,
    Surrender = 9,
    LeaveGame = 10,
    PlayAgain = 11,
}
