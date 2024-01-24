template="""
x-common-variables: &common-variables
  MONGOURI: "mongodb://${MONGO_LOGIN}:${MONGO_PASSWORD}@%container_namespace%-database:27017/?authSource=admin"
  JWT_SECRET: "${JWT_SECRET}"
  HELLO_MAIL_ADDRESS: "${HELLO_MAIL_ADDRESS}"
  HELLO_MAIL_PASSWORD: "${HELLO_MAIL_PASSWORD}"
  ADMIN_CREATION_PASSWORD: "${ADMIN_CREATION_PASSWORD}"
  AUDITORS_SERVICE_URL: "%AUDITORS_SERVICE_URL%"
  AUDITS_SERVICE_URL: "%AUDITS_SERVICE_URL%"
  CUSTOMERS_SERVICE_URL: "%CUSTOMERS_SERVICE_URL%"
  FILES_SERVICE_URL: "%FILES_SERVICE_URL%"
  MAIL_SERVICE_URL: "%MAIL_SERVICE_URL%"
  SEARCH_SERVICE_URL: "%SEARCH_SERVICE_URL%"
  USERS_SERVICE_URL: "%USERS_SERVICE_URL%"
  RENDERER_SERVICE_URL: "%RENDERER_SERVICE_URL%"
  NOTIFICATIONS_SERVICE_URL: "%NOTIFICATIONS_SERVICE_URL%"
  EVENTS_SERVICE_URL: "%EVENTS_SERVICE_URL%"
  API_PREFIX: "%API_PREFIX%"
  FRONTEND: "%FRONTEND%"
  PROTOCOL: "${PROTOCOL}"
  FEEDBACK_EMAIL: "${FEEDBACK_EMAIL}"
  GITHUB_CLIENT_SECRET: "${GITHUB_CLIENT_SECRET}"
  GITHUB_CLIENT_ID: "${GITHUB_CLIENT_ID}"
  RUST_LOG: actix=info,reqwest=info,search=info,common=info,audits=trace
  TIMEOUT: "60"



services:
  %container_namespace%-binaries:
    build:
      context: ./
      dockerfile: ./Dockerfile
    volumes:
      - %volume_namespace%-binaries:/data/binaries

  %container_namespace%-users:
    depends_on:
      - %container_namespace%-binaries
    build: ./users
    %port_expose%:
      - 3001%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(user|auth|my_user|waiting_list)
      <<: *common-variables
    networks:
      - %network_namespace%-database
      - %proxy_network%

  %container_namespace%-customers:
    depends_on:
      - %container_namespace%-binaries
    build: ./customers
    %port_expose%:
      - 3002%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(customer|my_customer|project|my_project)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-audits:
    depends_on:
      - %container_namespace%-binaries
    build: ./audits
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    %port_expose%:
      - 3003%optional_duplicate%
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(audit|my_audit|request|my_request|public_audits|no_customer_audit)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-auditors:
    depends_on:
      - %container_namespace%-binaries
    build: ./auditors
    %port_expose%:
      - 3004%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(auditor|my_auditor|badge)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-files:
    depends_on:
      - %container_namespace%-binaries
    build: ./files
    %port_expose%:
      - 3005%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
      - %volume_namespace%-files:/auditdb-files
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(file|notused1)
      <<: *common-variables
    networks:
      - %network_namespace%-database
      - %proxy_network%

  %container_namespace%-search:
    depends_on:
      - %container_namespace%-binaries
    build: ./search
    %port_expose%:
      - 3006%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(search|notused1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-mail:
    depends_on:
      - %container_namespace%-binaries
    build: ./mail
    %port_expose%:
      - 3007%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(mail|feedback|code)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-notification:
    depends_on:
      - %container_namespace%-binaries
    build: ./notification
    %port_expose%:
      - 3008%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(send_notification|read_notification|unread_notifications)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-telemetry:
    depends_on:
      - %container_namespace%-binaries
    build: ./telemetry
    %port_expose%:
      - 3009%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(telemetry|not_used1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-chat:
    depends_on:
      - %container_namespace%-binaries
    build: ./chat
    %port_expose%:
      - 3012%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(chat|not_used1)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-database

  %container_namespace%-event:
    depends_on:
      - %container_namespace%-binaries
    build: ./event
    %port_expose%:
      - 3010%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(notifications|event)
      <<: *common-variables
    networks:
      - %proxy_network%

  %container_namespace%-renderer:
    build:
      context: renderer
      dockerfile: Dockerfile
    %port_expose%:
      - 3015%optional_duplicate%
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(generate-report|notused2)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-report

  %container_namespace%-report:
    build: ./report
    depends_on:
      - %container_namespace%-binaries
      - %container_namespace%-renderer
    %port_expose%:
      - 3011%optional_duplicate%
    volumes:
      - %volume_namespace%-binaries:/data/binaries
    environment:
      VIRTUAL_HOST: "${VIRTUAL_HOST}"
      VIRTUAL_PATH: ~^/%API_PREFIX%/(report|notused2)
      <<: *common-variables
    networks:
      - %proxy_network%
      - %network_namespace%-report

  %container_namespace%-database:
    build: ./mongo
    environment:
      - MONGO_INITDB_ROOT_USERNAME=${MONGO_LOGIN}
      - MONGO_INITDB_ROOT_PASSWORD=${MONGO_PASSWORD}
    %open_database%:
      - 27017:27017
    volumes:
      - %volume_namespace%-database:/data/db
      - %volume_namespace%-backup:/mongo_backup
    networks:
      - %network_namespace%-database
volumes:
  %volume_namespace%-database:
  %volume_namespace%-backup:
  %volume_namespace%-files:
  %volume_namespace%-binaries:
networks:
  %network_namespace%-report:
  %network_namespace%-database:
  %proxy_network%:"""


dockerbuild = """
FROM rust:bookworm  as chef
RUN cargo install cargo-chef --locked
WORKDIR /app


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release %features%

RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
"""
"""
--features "test_server"
"""
import subprocess

def clone_database_from(source, config, destination = None):
    destination = destination or "audit-backend_%volume_namespace%-database".replace("%volume_namespace%", config['volume_namespace'])
    print(source)
    command = ["docker", "run", "--rm", "-it", "-v", f"{source}:/from", "-v", f"{destination}:/to", "ubuntu", "bash", "-c", "cd /from ; cp -av . /to"]
    print("running a command", *command)
    subprocess.run(command)

services = {
  "%AUDITORS_SERVICE_URL%": "auditors",
  "%AUDITS_SERVICE_URL%": "audits",
  "%CUSTOMERS_SERVICE_URL%": "customers",
  "%FILES_SERVICE_URL%": "files",
  "%MAIL_SERVICE_URL%": "mail",
  "%SEARCH_SERVICE_URL%": "search",
  "%USERS_SERVICE_URL%": "users",
  "%RENDERER_SERVICE_URL%": "renderer",
  "%NOTIFICATIONS_SERVICE_URL%": "notification",
  "%EVENTS_SERVICE_URL%": "event",
  "%FRONTEND%": "frontend"
}

# container_namespace, volume_namespace, network_namespace, load_database, open_database, with_proxy, export_database, import_database
def create_compose(config):
    global template, services
    template_instance = template
    port_expose = "ports"


    if config['open_database']:
        template_instance = template_instance.replace("%open_database%", "ports")
    else:
        template_instance = template_instance.replace("%open_database%", "expose")

    proxy_network_external = """
      external:
        true
    """
    proxy_newtwork_name = "nginx-proxy"

    if config['with_proxy']:
        port_expose = "expose"
        template_instance += proxy_network_external
        template_instance = template_instance.replace("%optional_duplicate%", "")
    else:
        while "%optional_duplicate%" in template_instance:
            pos = template_instance.find("%optional_duplicate%")
            template_instance = template_instance[:pos] + ":" + template_instance[pos-4:pos] + template_instance[pos + len("%optional_duplicate%"):]


    template_instance = template_instance.replace("%port_expose%", port_expose)
    
    template_instance = template_instance.replace("%API_PREFIX%", config["api_prefix"] if config["api_prefix"] else 'api')


    for pattern, key in services.items():
        value = config[key] if key in config else config["proxy_address"]
        
        template_instance = template_instance.replace(pattern, value)

    template_instance = template_instance.replace("%proxy_network%", proxy_newtwork_name)

    template_instance = template_instance.replace("%container_namespace%", config['container_namespace'])
    template_instance = template_instance.replace("%volume_namespace%", config['volume_namespace'])
    template_instance = template_instance.replace("%network_namespace%", config['network_namespace'])
    return template_instance

def create_docker(config):
    global dockerbuild
    dockerbuild_instance = dockerbuild
    features = ("--features " + config['features']) if 'features' in config else ""
    dockerbuild_instance = dockerbuild_instance.replace("%features%", features)
    return dockerbuild_instance


preset = {
    "dev": {
        "open_database": True,
        "with_proxy": False,
        "container_namespace": "dev",
        "volume_namespace": "dev",
        "network_namespace": "dev",
        "auditors": "0.0.0.0:3004",
        "audits": "0.0.0.0:3003",
        "chat": "0.0.0.0:3012",
        "customers": "0.0.0.0:3002",
        "event": "0.0.0.0:3010",
        "files": "0.0.0.0:3005",
        "mail": "0.0.0.0:3007",
        "notification": "0.0.0.0:3008",
        "renderer": "0.0.0.0:3015",
        "report": "0.0.0.0:3011",
        "search": "0.0.0.0:3006",
        "telemetry": "0.0.0.0:3009",
        "users": "0.0.0.0:3001",
        "frontend": "dev.auditdb.io",
        "api_prefix": "api"
    },

    "prod": {
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "prod",
        "volume_namespace": "prod",
        "network_namespace": "prod",
        "proxy_address": "auditdb.io",
        "api_prefix": "api"

    },
    "test": {
        "open_database": True,
        "with_proxy": True,
        "container_namespace": "test",
        "volume_namespace": "test",
        "network_namespace": "test",
        "proxy_address": "dev.auditdb.io",
        "features": '"test_server"',
        "api_prefix": "api"


    },
    "preprod": {
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "preprod",
        "volume_namespace": "preprod",
        "network_namespace": "preprod",
        "proxy_address": "preprod.auditdb.io",
        "api_prefix": "api"
    }
}


from dotenv import load_dotenv
import os


def get_config():
    global preset


    config = {
        "open_database": os.getenv("OPEN_DATABASE"), 
        "with_proxy": os.getenv("WITH_PROXY"), 
        "container_namespace": os.getenv("CONTAINER_NAMESPACE"), 
        "volume_namespace": os.getenv("VOLUME_NAMESPACE"), 
        "network_namespace": os.getenv("NETWORK_NAMESPACE"),
        "api_prefix": os.getenv("API_PREFIX")
    }
    
    if os.environ["PRESET"] is not None:
        preset_config = preset[os.environ["PRESET"]]
        for key, value in config.items():
            if preset_config[key] is not None and value is not None:
              preset_config[key] = value 
        return preset_config
    return config
    
import sys

def main():
    load_dotenv()

    config = get_config()

    
    if len(sys.argv) > 1:
        clone_database_from(sys.argv[1], config)
        return

    compose = create_compose(config)
    docker = create_docker(config)

    
    with open("docker-compose.yml", "w") as f:
        f.write(compose)

    with open("Dockerfile", "w") as f:
        f.write(docker)

if __name__ == "__main__":
    main()