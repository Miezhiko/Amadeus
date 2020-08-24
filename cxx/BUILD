cc_library(
  name = "moth-cxx",
  srcs = ["moth.cc"],
  copts = ["-std=c++17"],
  visibility = ["//visibility:public"],
  deps = [
    ":include",
    "//moth-rs:include",
  ],
)

cc_library(
  name = "include",
  hdrs = ["moth.hpp"],
  visibility = ["//visibility:public"],
  deps = ["//:core"],
)
