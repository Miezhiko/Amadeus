let Streams : Type = ./streams_type.dhall
in { battletag: Text
   , discord: Natural
   , streams: Optional Streams
   , alt_accounts: List Text }
