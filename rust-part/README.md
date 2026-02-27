1. Axum for handling routing
2. Tokio for Axum dependency and Async handling
3. axum-server and rustls-pemfile for loading PEM files, handling https
4. Downloaded mkcert for certificate generation, and https
5. mkcert -install and mkcert localhost to create certificates valid for those endpoints
6. 