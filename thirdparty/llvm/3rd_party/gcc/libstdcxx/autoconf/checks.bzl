# Small local autoconf-style check API.
#
# This file intentionally mirrors the shape of external autoconf rulesets:
# source-counterpart files declare checks as JSON-encoded data, execution rules
# consume those declarations, and header rules render the results. Keep this
# generic and free of libstdc++ policy.

def _make_check(fields):
    return json.encode(fields)

def _language(language):
    if language == "cpp":
        return "c++"
    return language

def compile_check(name, source, language = "c", flags = [], defines_on_success = None, probe_contexts = []):
    if defines_on_success == None:
        defines_on_success = [name]
    return _make_check({
        "defines_on_success": defines_on_success,
        "flags": flags,
        "language": _language(language),
        "name": name,
        "probe_contexts": probe_contexts,
        "source": source.strip() + "\n",
        "type": "compile",
    })

def link_check(name, source, language = "c++", compile_flags = [], link_flags = [], defines_on_success = None, probe_contexts = []):
    if defines_on_success == None:
        defines_on_success = [name]
    return _make_check({
        "compile_flags": compile_flags,
        "defines_on_success": defines_on_success,
        "language": _language(language),
        "link_flags": link_flags,
        "name": name,
        "probe_contexts": probe_contexts,
        "source": source.strip() + "\n",
        "type": "link",
    })

def policy_define(name, value = "1", defines_on_success = None):
    if defines_on_success == None:
        defines_on_success = [name]
    return _make_check({
        "defines_on_success": defines_on_success,
        "name": name,
        "type": "define",
        "value": str(value),
    })

def policy_undef(name):
    return _make_check({
        "name": name,
        "type": "undef",
    })

def policy_string_define(name, value):
    return _make_check({
        "name": name,
        "type": "string_define",
        "value": value,
    })

def header_check(header):
    return compile_check(
        name = "HAVE_" + header.upper().replace("/", "_").replace(".", "_"),
        source = """
#include <{header}>
int main(void) {{ return 0; }}
""".format(header = header),
    )

def ac_check_headers(headers):
    return [header_check(header) for header in headers]

def function_link_check(name, header, expression, language = "c++", compile_flags = [], link_flags = [], probe_contexts = []):
    return link_check(
        name = name,
        language = language,
        compile_flags = compile_flags,
        link_flags = link_flags,
        probe_contexts = probe_contexts,
        source = """
#include <{header}>
int main() {{
    {expression};
    return 0;
}}
""".format(header = header, expression = expression),
    )

def AC_TRY_COMPILE(name, code, language = "c", CFLAGS = [], CXXFLAGS = [], define = None, probe_contexts = []):
    return compile_check(
        name = name,
        source = code,
        language = language,
        flags = CXXFLAGS if _language(language) == "c++" else CFLAGS,
        defines_on_success = [define or name],
        probe_contexts = probe_contexts,
    )

def AC_TRY_LINK(name, code, language = "c++", CFLAGS = [], CXXFLAGS = [], LIBS = [], define = None, probe_contexts = []):
    return link_check(
        name = name,
        source = code,
        language = language,
        compile_flags = CXXFLAGS if _language(language) == "c++" else CFLAGS,
        link_flags = LIBS,
        defines_on_success = [define or name],
        probe_contexts = probe_contexts,
    )

def AC_DEFINE(name, value = "1"):
    return policy_define(name, value)

def AC_FAIL(name):
    return policy_undef(name)

def AC_CHECK_HEADERS(headers):
    return ac_check_headers(headers)

checks = struct(
    AC_CHECK_HEADERS = AC_CHECK_HEADERS,
    AC_DEFINE = AC_DEFINE,
    AC_FAIL = AC_FAIL,
    AC_TRY_COMPILE = AC_TRY_COMPILE,
    AC_TRY_LINK = AC_TRY_LINK,
)
