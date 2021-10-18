//debug:
//import rep from "../node_modules/w3gjs";

import rep from "/usr/lib64/node_modules/w3gjs";
import { units, heroes } from "./mappings"

const replay = "./" + process.argv[2];

const highlevel_parser = new rep();

type LeaverResult = {
  id: number
  reason: string
  result: string
}

var leaver_results: LeaverResult[] = [];

highlevel_parser.on("gamedatablock", (block) => {
  if (block.id === 0x17) {
    const leaver_result: LeaverResult =
      { id: block.playerId
      , reason: block.reason
      , result: block.result };
    leaver_results.push(leaver_result);
  }
});

highlevel_parser
  .parse(replay)
  .then((result) => {
    for (let playa of result.players) {
      var new_dict: { [id: string]: number; } = {};
      for (let key in playa.units.summary) {
        let count = playa.units.summary[key];
        let new_key = units[key].substring(2);
        new_dict[new_key] = count;
      }
      playa.units.summary = new_dict;
      for (let hero of playa.heroes) {
        let new_id = heroes[hero.id];
        hero.id = new_id;
      }
    }
    if (result.players.length > 1
     && leaver_results.length > 1) {
      var found = false;
      for (let i in leaver_results) {
        if (leaver_results[i].result == "0b000000") {
          for (let j in leaver_results) {
            if (leaver_results[i].id == result.players[j].id) {
              found = true;
              result.map.checksum = result.players[j].name;
              break;
            }
          }
        }
      }
      if (!found) {
        result.map.checksum = "";
      }
    }
    console.log(
      JSON.stringify(result, null, '  ')
    );
  })
  .catch(console.error);
