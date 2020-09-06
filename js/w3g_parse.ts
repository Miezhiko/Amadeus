import rep from "/usr/lib64/node_modules/w3gjs";
//node node_modules/ts-node/dist/bin.js js/w3g_parse.ts LastReplay.w3g
//import rep from "w3gjs";
//import { ParserOutput } from "/usr/lib64/node_modules/w3gjs/dist/types/types";

const replay = "./" + process.argv[2];

/*class Customized extends rep {
  constructor() {
    super();
  }
  finalize(): ParserOutput {
    const result = super.finalize();
    console.log(this.leaveEvents);
    if (result.players.length > 1
     && this.leaveEvents.length > 1) {
      const p1 = {
        id: result.players[0].id,
        name: result.players[0].name,
      };
      const p2 = {
        id: result.players[1].id,
        name: result.players[1].name,
      };
      result.map.checksum = "...";
    }
    return result;
  }
}*/

const highlevel_parser = new rep();

var leaver_results = [];

highlevel_parser.on("gamedatablock", (block) => {
  if (block.id === 0x17) {
    const leaver_result = 
      { id: block.playerId
      , reason: block.reason
      , result: block.result };
    leaver_results.push(leaver_result);
  }
});

highlevel_parser
  .parse(replay)
  .then((result) => {
    if (result.players.length > 1
     && leaver_results.length > 1) {
      // trying to analyse result from saver (last leaver)
      if (leaver_results[1].result.toString(16) == '0d000000') {
        if (leaver_results[1].id == result.players[0].id) {
          result.map.checksum = result.players[1].name;
        } else {
          result.map.checksum = result.players[0].name;
        }
      } else if (leaver_results[1].result == 0b000000) {
        if (leaver_results[1].id == result.players[0].id) {
          result.map.checksum = result.players[0].name;
        } else {
          result.map.checksum = result.players[1].name;
        }
      } else {
        result.map.checksum = "";
      }
    }
    console.log(
      JSON.stringify(result, null, '  ')
    );
  })
  .catch(console.error);
