# OCI image for botifactory, published by .gitea/workflows/container.yml.
# A drop-in replacement for the old Dockerfile image, so docker-compose.yml
# works unchanged:
#
#  - The app reads ./configuration relative to its working directory, so the
#    image runs in /app with configuration/ linked in from the package.
#  - /releases and /data are the volume mount points for release binaries and
#    the SQLite database (see configuration/production.toml).
#  - It runs as root, like the Dockerfile image did: existing volumes hold
#    root-owned files that an unprivileged user could not write.
#
# BOTIFACTORY_APP__APPLICATION__API_TOKEN must be supplied at run time; the
# production config refuses to start without it.
{
  lib,
  dockerTools,
  cacert,
  botifactory,
}:

dockerTools.buildLayeredImage {
  name = "botifactory";
  tag = "latest";

  contents = [ cacert ];

  extraCommands = ''
    mkdir -p app releases data tmp
    chmod 1777 tmp
    ln -s ${botifactory}/share/botifactory/configuration app/configuration
  '';

  config = {
    Entrypoint = [ (lib.getExe botifactory) ];
    WorkingDir = "/app";
    ExposedPorts."8000/tcp" = { };
    Volumes = {
      "/releases" = { };
      "/data" = { };
    };
    Env = [
      "BOTIFACTORY_APP_ENVIRONMENT=production"
      "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
    ];
  };
}
