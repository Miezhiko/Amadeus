let Player : Type = ./player_type.dhall
in { uid: Natural
   , games: Optional Natural
   , games2: Optional Natural
   , games4: Optional Natural
   , streams: Optional Natural
   , events: Optional Natural
   , players: List Player
   }
