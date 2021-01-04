(function() {var implementors = {};
implementors["spartan"] = [{"text":"impl Send for ResponseError","synthetic":true,"types":[]},{"text":"impl Send for QueueError","synthetic":true,"types":[]},{"text":"impl Send for StartCommandError","synthetic":true,"types":[]},{"text":"impl Send for StartCommand","synthetic":true,"types":[]},{"text":"impl Send for InitCommandError","synthetic":true,"types":[]},{"text":"impl Send for InitCommand","synthetic":true,"types":[]},{"text":"impl Send for ReplicaCommand","synthetic":true,"types":[]},{"text":"impl Send for Command","synthetic":true,"types":[]},{"text":"impl Send for Server","synthetic":true,"types":[]},{"text":"impl Send for ManagerError","synthetic":true,"types":[]},{"text":"impl&lt;'c&gt; Send for Manager&lt;'c&gt;","synthetic":true,"types":[]},{"text":"impl&lt;DB&gt; Send for Queue&lt;DB&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;DB: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'c&gt; Send for Log&lt;'c&gt;","synthetic":true,"types":[]},{"text":"impl Send for PersistMode","synthetic":true,"types":[]},{"text":"impl&lt;'c&gt; Send for Snapshot&lt;'c&gt;","synthetic":true,"types":[]},{"text":"impl Send for PersistenceError","synthetic":true,"types":[]},{"text":"impl&lt;'msg&gt; Send for Event&lt;'msg&gt;","synthetic":true,"types":[]},{"text":"impl Send for ReplicationStorage","synthetic":true,"types":[]},{"text":"impl&lt;'c, 'r&gt; Send for PrimaryRequest&lt;'c, 'r&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'c&gt; Send for ReplicaRequest&lt;'c&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'c, 'r&gt; Send for Request&lt;'c, 'r&gt;","synthetic":true,"types":[]},{"text":"impl Send for PrimaryError","synthetic":true,"types":[]},{"text":"impl&lt;'s, T&gt; Send for RecvIndex&lt;'s, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'s, T&gt; Send for BatchAskIndex&lt;'s, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'s, T&gt; Send for Sync&lt;'s, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Send for Stream&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Send for StreamPool&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for PrimaryStorage","synthetic":true,"types":[]},{"text":"impl Send for ReplicaError","synthetic":true,"types":[]},{"text":"impl Send for ReplicaStorage","synthetic":true,"types":[]},{"text":"impl&lt;'m, 'c, T&gt; Send for ReplicaSocket&lt;'m, 'c, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'c, S&gt; Send for Node&lt;'c, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for DeleteRequest","synthetic":true,"types":[]},{"text":"impl Send for DeleteResponse","synthetic":true,"types":[]},{"text":"impl&lt;'m&gt; Send for Timeout&lt;'m&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'m&gt; Send for Time&lt;'m&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'m&gt; Send for PopResponse&lt;'m&gt;","synthetic":true,"types":[]},{"text":"impl Send for PushRequest","synthetic":true,"types":[]},{"text":"impl Send for RequeueRequest","synthetic":true,"types":[]},{"text":"impl Send for SizeResponse","synthetic":true,"types":[]},{"text":"impl Send for ServerError","synthetic":true,"types":[]},{"text":"impl Send for AccessError","synthetic":true,"types":[]},{"text":"impl Send for AccessMiddleware","synthetic":true,"types":[]},{"text":"impl Send for Key","synthetic":true,"types":[]},{"text":"impl Send for Primary","synthetic":true,"types":[]},{"text":"impl Send for Replica","synthetic":true,"types":[]},{"text":"impl Send for Replication","synthetic":true,"types":[]},{"text":"impl Send for ReplicationConfig","synthetic":true,"types":[]},{"text":"impl Send for Persistence","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Send for PersistenceConfig&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Send for Config&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Send for BincodeCodec","synthetic":true,"types":[]}];
implementors["spartan_lib"] = [{"text":"impl&lt;M&gt; Send for TreeDatabase&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Identifiable&gt;::Id: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Sortable&gt;::Sort: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Send for VecDatabase&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for BuilderError","synthetic":true,"types":[]},{"text":"impl Send for MessageBuilder","synthetic":true,"types":[]},{"text":"impl Send for Timeout","synthetic":true,"types":[]},{"text":"impl Send for Offset","synthetic":true,"types":[]},{"text":"impl Send for Time","synthetic":true,"types":[]},{"text":"impl Send for Status","synthetic":true,"types":[]},{"text":"impl Send for State","synthetic":true,"types":[]},{"text":"impl Send for Message","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()