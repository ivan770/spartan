(function() {var implementors = {};
implementors["spartan"] = [{"text":"impl&lt;'msg&gt; Serialize for Event&lt;'msg&gt;","synthetic":false,"types":[]},{"text":"impl Serialize for ReplicationStorage","synthetic":false,"types":[]},{"text":"impl&lt;'c, 'r&gt; Serialize for PrimaryRequest&lt;'c, 'r&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'c&gt; Serialize for ReplicaRequest&lt;'c&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'c, 'r&gt; Serialize for Request&lt;'c, 'r&gt;","synthetic":false,"types":[]},{"text":"impl Serialize for PrimaryStorage","synthetic":false,"types":[]},{"text":"impl Serialize for ReplicaStorage","synthetic":false,"types":[]},{"text":"impl Serialize for DeleteResponse","synthetic":false,"types":[]},{"text":"impl&lt;'m&gt; Serialize for Timeout&lt;'m&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'m&gt; Serialize for Time&lt;'m&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'m&gt; Serialize for PopResponse&lt;'m&gt;","synthetic":false,"types":[]},{"text":"impl Serialize for SizeResponse","synthetic":false,"types":[]},{"text":"impl Serialize for Key","synthetic":false,"types":[]},{"text":"impl Serialize for Primary","synthetic":false,"types":[]},{"text":"impl Serialize for Replica","synthetic":false,"types":[]},{"text":"impl Serialize for Replication","synthetic":false,"types":[]},{"text":"impl Serialize for ReplicationConfig","synthetic":false,"types":[]},{"text":"impl Serialize for Persistence","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Serialize for PersistenceConfig&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Serialize for Config&lt;'a&gt;","synthetic":false,"types":[]}];
implementors["spartan_lib"] = [{"text":"impl&lt;M&gt; Serialize for TreeDatabase&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Identifiable + Sortable,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Identifiable&gt;::Id: Hash,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Identifiable&gt;::Id: Serialize + DeserializeOwned,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Sortable&gt;::Sort: Serialize + DeserializeOwned,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Serialize + DeserializeOwned,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M&gt; Serialize for VecDatabase&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Serialize,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Serialize for Timeout","synthetic":false,"types":[]},{"text":"impl Serialize for Offset","synthetic":false,"types":[]},{"text":"impl Serialize for Time","synthetic":false,"types":[]},{"text":"impl Serialize for Status","synthetic":false,"types":[]},{"text":"impl Serialize for State","synthetic":false,"types":[]},{"text":"impl Serialize for Message","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()