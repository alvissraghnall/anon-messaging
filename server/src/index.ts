import Database from "./common/database";
import Server from "./common/server";
import routes from "./routes";
import io, { Server as SocketIOServer } from 'socket.io'
import { ChatEvent } from "./common/socket-constants";
import l from "./common/logger";

const port = parseInt(process.env.PORT || "3000");
const connectionString = process.env.MONGODB_URI;
  // process.env.NODE_ENV === "production"
  //   ? process.env.MONGODB_URI
  //   : process.env.NODE_ENV === "test"
  //   ? process.env.MONGODB_URI_TEST ||
  //     "mongodb://localhost:27017/anon-messaging"
  //   : process.env.MONGODB_URI_DEV ||
  //     "mongodb://localhost:27017/anon-messaging";

const db = new Database(connectionString);
/*export default */const server = new Server().database(db).router(routes).listen(port);

const sockio = new SocketIOServer(server);
sockio.on(ChatEvent.CONNECT, (socket) => {
  l.info(`Connected on port %s.`, port);
  
  socket.on(ChatEvent.MESSAGE, () => {


  })

  socket.on(ChatEvent.DISCONNECT, () => {
    l.info("Client disconnected");
  })
})
