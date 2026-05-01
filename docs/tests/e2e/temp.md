Checking GraphDB server at 127.0.0.1:9758...
✓ Server is ready

============================================================
Running Test Suite: Schema Manager Init
============================================================
test_001_basic_connection (test_schema_manager_init.TestSchemaManagerInitialization.test_001_basic_connection)
TC-SCHEMA-001: Verify basic connection works. ... ok
test_002_create_space_without_vector (test_schema_manager_init.TestSchemaManagerInitialization.test_002_create_space_without_vector)
TC-SCHEMA-002: Create space should work regardless of vector config. ... ok
test_003_use_space (test_schema_manager_init.TestSchemaManagerInitialization.test_003_use_space)  
TC-SCHEMA-003: Use space should work. ... ok
test_004_create_tag (test_schema_manager_init.TestSchemaManagerInitialization.test_004_create_tag)
TC-SCHEMA-004: Create tag should work with schema_manager. ... ok
test_005_show_tags (test_schema_manager_init.TestSchemaManagerInitialization.test_005_show_tags)  
TC-SCHEMA-005: Show tags should work. ... ok
test_006_insert_vertex (test_schema_manager_init.TestSchemaManagerInitialization.test_006_insert_vertex)
TC-SCHEMA-006: Insert vertex should work. ... ok
test_007_fetch_vertex (test_schema_manager_init.TestSchemaManagerInitialization.test_007_fetch_vertex)
TC-SCHEMA-007: Fetch vertex should work. ... ok
test_008_match_query (test_schema_manager_init.TestSchemaManagerInitialization.test_008_match_query)
TC-SCHEMA-008: MATCH query should work. ... ok
test_009_drop_space (test_schema_manager_init.TestSchemaManagerInitialization.test_009_drop_space)
TC-SCHEMA-009: Drop space should work. ... ok
test_error_message_clarity (test_schema_manager_init.TestSchemaManagerErrorHandling.test_error_message_clarity)
TC-SCHEMA-ERR-001: Error messages should be clear when operations fail. ... ok
test_show_spaces_always_works (test_schema_manager_init.TestSchemaManagerErrorHandling.test_show_spaces_always_works)
TC-SCHEMA-ERR-002: SHOW SPACES should always work. ... ok

---

Ran 11 tests in 0.117s

OK

============================================================
Running Test Suite: Social Network
============================================================
test_001_connect_and_show_spaces (test_social_network.TestSocialNetworkBasic.test_001_connect_and_show_spaces)
TC-001: Connect to server and list spaces. ... ok
test_002_create_and_use_space (test_social_network.TestSocialNetworkBasic.test_002_create_and_use_space)
TC-002: Create space and switch to it. ... ok
test_003_create_tags_and_edges (test_social_network.TestSocialNetworkBasic.test_003_create_tags_and_edges)
TC-003: Create tags and edge types. ... ok
test_004_show_tags (test_social_network.TestSocialNetworkBasic.test_004_show_tags)
TC-004: Verify tags were created. ... ok
test_005_show_edges (test_social_network.TestSocialNetworkBasic.test_005_show_edges)
TC-005: Verify edges were created. ... ok
test_006_insert_vertex (test_social_network.TestSocialNetworkData.test_006_insert_vertex)
TC-006: Insert vertex data. ... ok
test_007_insert_multiple_vertices (test_social_network.TestSocialNetworkData.test_007_insert_multiple_vertices)
TC-007: Insert multiple vertices. ... ok
test_008_insert_edge (test_social_network.TestSocialNetworkData.test_008_insert_edge)
TC-008: Insert edge data. ... ok
test_009_fetch_vertex (test_social_network.TestSocialNetworkData.test_009_fetch_vertex)
TC-009: Fetch vertex properties. ... ok
test_010_fetch_edge (test_social_network.TestSocialNetworkData.test_010_fetch_edge)
TC-010: Fetch edge properties. ... ok
test_011_match_basic (test_social_network.TestSocialNetworkQueries.test_011_match_basic)
TC-011: Basic MATCH query. ... ok
test_012_match_with_filter (test_social_network.TestSocialNetworkQueries.test_012_match_with_filter)
TC-012: MATCH with filter condition. ... ok
test_013_match_path (test_social_network.TestSocialNetworkQueries.test_013_match_path)
TC-013: MATCH path query. ... ok
test_014_go_traversal (test_social_network.TestSocialNetworkQueries.test_014_go_traversal)  
TC-014: GO traversal query. ... ok
test_015_go_multiple_steps (test_social_network.TestSocialNetworkQueries.test_015_go_multiple_steps)
TC-015: GO multi-step traversal. ... ok
test_016_lookup_index (test_social_network.TestSocialNetworkQueries.test_016_lookup_index)  
TC-016: LOOKUP index query. ... ok
test_017_explain_basic (test_social_network.TestSocialNetworkExplain.test_017_explain_basic)
TC-017: Basic EXPLAIN query. ... ok
test_018_explain_with_index (test_social_network.TestSocialNetworkExplain.test_018_explain_with_index)
TC-018: EXPLAIN with index scan. ... ok
test_019_profile_query (test_social_network.TestSocialNetworkExplain.test_019_profile_query)  
TC-019: PROFILE query execution. ... ok
test_020_transaction_commit (test_social_network.TestSocialNetworkTransaction.test_020_transaction_commit)
TC-020: Basic transaction commit. ... FAIL
test_021_transaction_rollback (test_social_network.TestSocialNetworkTransaction.test_021_transaction_rollback)
TC-021: Transaction rollback. ... ERROR
setUpClass (test_social_network.TestSocialNetworkCleanup) ... ERROR

======================================================================
ERROR: test_021_transaction_rollback (test_social_network.TestSocialNetworkTransaction.test_021_transaction_rollback)
TC-021: Transaction rollback.

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_social_network.py", line 416, in setUp
self.client.connect()
~~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_social_network.TestSocialNetworkCleanup)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_social_network.py", line 491, in setUpClass  
 cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
FAIL: test_020_transaction_commit (test_social_network.TestSocialNetworkTransaction.test_020_transaction_commit)
TC-020: Basic transaction commit.

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_social_network.py", line 446, in test_020_transaction_commit
self.assertTrue(result.success, f"INSERT failed: {result.error}")
~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
AssertionError: False is not true : INSERT failed: Request timeout after 30s

---

Ran 21 tests in 131.572s

FAILED (failures=1, errors=2)

============================================================
Running Test Suite: Optimizer
============================================================
setUpClass (test_optimizer.TestOptimizerIndex) ... ERROR
setUpClass (test_optimizer.TestOptimizerJoin) ... ERROR
setUpClass (test_optimizer.TestOptimizerAggregate) ... ERROR
setUpClass (test_optimizer.TestOptimizerTopN) ... ERROR
setUpClass (test_optimizer.TestOptimizerExplainFormat) ... ERROR
setUpClass (test_optimizer.TestOptimizerProfile) ... ERROR
setUpClass (test_optimizer.TestOptimizerCleanup) ... ERROR

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerIndex)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 27, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerJoin)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 114, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerAggregate)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 188, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerTopN)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 241, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerExplainFormat)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 291, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerProfile)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 336, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_optimizer.TestOptimizerCleanup)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 378, in setUpClass
cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

---

Ran 0 tests in 210.162s

FAILED (errors=7)

============================================================
Running Test Suite: Extended Types
============================================================
setUpClass (test_extended_types.TestGeography) ... ERROR
setUpClass (test_extended_types.TestVector) ... ERROR
setUpClass (test_extended_types.TestFullText) ... ERROR
setUpClass (test_extended_types.TestExtendedTypesCleanup) ... ERROR

======================================================================
ERROR: setUpClass (test_extended_types.TestGeography)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_extended_types.py", line 25, in setUpClass  
 cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_extended_types.TestVector)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_extended_types.py", line 159, in setUpClass  
 cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_extended_types.TestFullText)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_extended_types.py", line 273, in setUpClass  
 cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

======================================================================
ERROR: setUpClass (test_extended_types.TestExtendedTypesCleanup)

---

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 534, in \_make_request
response = conn.getresponse()
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connection.py", line 571, in getresponse
httplib_response = super().getresponse()
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 1430, in getresponse
response.begin()
~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 331, in begin
version, status, reason = self.\_read_status()
~~~~~~~~~~~~~~~~~^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\http\client.py", line 292, in \_read_status
line = str(self.fp.readline(\_MAXLINE + 1), "iso-8859-1")
~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^
File "C:\Users\33530\AppData\Roaming\uv\python\cpython-3.14.0-windows-x86_64-none\Lib\socket.py", line 725, in readinto
return self.\_sock.recv_into(b)
~~~~~~~~~~~~~~~~~~~~^^^
TimeoutError: timed out

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 645, in send
resp = conn.urlopen(
method=request.method,
...<9 lines>...
chunked=chunked,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 841, in urlopen
retries = retries.increment(
method, url, error=new_e, \_pool=self, \_stacktrace=sys.exc_info()[2]
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\retry.py", line 490, in increment
raise reraise(type(error), error, \_stacktrace)
~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\util\util.py", line 39, in reraise
raise value
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 787, in urlopen
response = self.\_make_request(
conn,
...<10 lines>...
\*\*response_kw,
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 536, in \_make_request
self.\_raise_timeout(err=e, url=url, timeout_value=read_timeout)
~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\urllib3\connectionpool.py", line 367, in \_raise_timeout
raise ReadTimeoutError(
self, url, f"Read timed out. (read timeout={timeout_value})"
) from err
urllib3.exceptions.ReadTimeoutError: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
File "D:\项目\database\graphDB\tests\e2e\test_extended_types.py", line 372, in setUpClass  
 cls.client.connect()
~~~~~~~~~~~~~~~~~~^^
File "D:\项目\database\graphDB\tests\e2e\graphdb_client.py", line 56, in connect
response = self.session.get(
f"{self.base_url}/v1/health",
timeout=self.timeout
)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 605, in get
return self.request("GET", url, \*\*kwargs)
~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 592, in request
resp = self.send(prep, **send_kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\sessions.py", line 706, in send
r = adapter.send(request, **kwargs)
File "D:\项目\database\graphDB\tests\e2e\.venv\Lib\site-packages\requests\adapters.py", line 691, in send
raise ReadTimeout(e, request=request)
requests.exceptions.ReadTimeout: HTTPConnectionPool(host='127.0.0.1', port=9758): Read timed out. (read timeout=30)

---

Ran 0 tests in 120.102s

FAILED (errors=4)

============================================================
TEST SUMMARY
============================================================

✓ PASS - Schema Manager Init
Total: 11
Passed: 11
Failed: 0
Errors: 0
Skipped: 0

✗ FAIL - Social Network
Total: 21
Passed: 18
Failed: 1
Errors: 2
Skipped: 0

✗ FAIL - Optimizer
Total: 0
Passed: -7
Failed: 0
Errors: 7
Skipped: 0

✗ FAIL - Extended Types
Total: 0
Passed: -4
Failed: 0
Errors: 4
Skipped: 0

---

## OVERALL

Total Tests: 32
Passed: 18
Failed: 1
Errors: 13
Skipped: 0
Duration: 462.00s
