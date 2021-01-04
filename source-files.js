var N = null;var sourcesIndex = {};
sourcesIndex["spartan"] = {"name":"","dirs":[{"name":"actions","files":["clear.rs","delete.rs","mod.rs","pop.rs","push.rs","requeue.rs","size.rs"]},{"name":"cli","dirs":[{"name":"commands","files":["init.rs","mod.rs","replica.rs","start.rs"]}],"files":["mod.rs"]},{"name":"config","files":["key.rs","mod.rs","persistence.rs","replication.rs"]},{"name":"http","dirs":[{"name":"middleware","files":["access.rs","mod.rs"]},{"name":"query","files":["delete.rs","mod.rs","pop.rs","push.rs","requeue.rs","size.rs"]}],"files":["mod.rs","routing.rs","server.rs"]},{"name":"jobs","files":["gc.rs","mod.rs","persistence.rs","replication.rs"]},{"name":"node","dirs":[{"name":"persistence","dirs":[{"name":"log","files":["mod.rs"]},{"name":"snapshot","files":["mod.rs"]}],"files":["mod.rs"]},{"name":"replication","dirs":[{"name":"primary","files":["error.rs","index.rs","mod.rs","storage.rs","stream.rs"]},{"name":"replica","files":["error.rs","mod.rs","storage.rs"]}],"files":["message.rs","mod.rs","storage.rs"]}],"files":["event.rs","manager.rs","mod.rs","queue.rs"]},{"name":"utils","files":["codec.rs","mod.rs"]}],"files":["main.rs"]};
sourcesIndex["spartan_lib"] = {"name":"","dirs":[{"name":"core","dirs":[{"name":"db","dirs":[{"name":"tree","files":["mod.rs"]},{"name":"vec","files":["mod.rs"]}],"files":["mod.rs"]},{"name":"dispatcher","files":["mod.rs","simple.rs","status_aware.rs"]},{"name":"message","files":["builder.rs","mod.rs","state.rs","time.rs"]},{"name":"payload","files":["dispatchable.rs","identifiable.rs","mod.rs","sortable.rs","status.rs"]}],"files":["mod.rs"]}],"files":["lib.rs"]};
createSourceSidebar();