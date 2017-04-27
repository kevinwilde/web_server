# web_server

## Overview

This is a basic web server implemented in Rust. It responds to GET requests which must
take the shape GET /path/to/file HTTP.  It also accepts newer HTTP versions, which will 
end their request with a token that includes the version, e.g., HTTP/1.1. The server 
listens on localhost (127.0.0.1), port 8080.

The possible responses are:
* 200 OK, which starts a reply that serves the specified file
* 400 Bad Request, which indicates that the command is not a properly formatted GET command
* 403 Forbidden, which rejects a command because it specifies a file that is off-limits
* 404 Not Found, which informs the client that the specified file does not exist

The path to the file must begin with / and is interpreted after concatenating it with 
the server’s root path.
* If the resulting path points to a file, the file is served with a 200 OK response 
unless its permissions do not allow so.
* If the resulting path points to a directory, it is interpreted as pointing to one of 
these files: index.html, index.shtml, and index.txt. The first file found is served 
assuming it is accessible. Otherwise the path triggers a 404-message.
* Otherwise the server responds with an error message.

## Assumptions
* The method token in an HTTP request is case sensitive, in accordance with 
[the information here](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html#sec5.1.1). 
So in the request, the word GET must be in all caps in order for it to be considered a 
valid request.
* A path to a directory should end with a /. If it does not, the server will still be 
able to find index.html, index.shtml, or index.txt, but in the case of an html file with 
relative links to external resources, those relative links will be broken. This is in 
accordance with [the information here](http://serverfault.com/questions/587002/apache2-301-redirect-when-missing-at-the-end-of-directory-in-the-url) 
(this server does not implement the response code 301). When the path to a directory 
ends with a / as it should, the server is of course able to resolve relative links. 
