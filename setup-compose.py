from typing import Optional, List

class PortConfig:
    expose: str
    port: str

    def __init__(self, expose, port) -> None:
      self.expose = expose
      self.port = port

class APIConfig:
    api_prefix: str
    post_prefixes: List[str]

    def __init__(self, api_prefix, post_prefixes) -> None:
        self.api_prefix = api_prefix
        self.post_prefixes = post_prefixes


def get_services(config, api_prefix, proxy_network, expose, open_database):
    open_database = "ports" if open_database else "expose" 
    default_service_settings = {
        'depends_on': ["binaries"],
        'volumes': [("binaries", "/data/binaries")],
        'networks': [proxy_network, "database"],
        'proxy_network': proxy_network
    }
    return [
        Service(config, "binaries", None, None, [], [("binaries", "/data/binaries")], [], proxy_network),
        Service(config, "users", PortConfig(expose, "3001"), APIConfig(api_prefix, ["user", "auth", "my_user", "waiting_list"]), **default_service_settings),
        Service(config, "customers", PortConfig(expose, "3002"), APIConfig(api_prefix, ["customer", "my_customer", "project", "my_project"]), **default_service_settings),
        Service(config, "audits", PortConfig(expose, "3003"), APIConfig(api_prefix, ["audit", "my_audit", "request", "my_request", "public_audits", "no_customer_audit"]), **default_service_settings),
        Service(config, "auditors", PortConfig(expose, "3004"), APIConfig(api_prefix, ["auditor", "my_auditor", "badge"]), **default_service_settings),
        Service(config, "files", PortConfig(expose, "3005"), APIConfig(api_prefix, ["file", "notused1"]), ["binaries"], [("binaries", "/data/binaries"), ("files", "/auditdb-files")], [proxy_network, "database"], proxy_network),
        Service(config, "search", PortConfig(expose, "3006"), APIConfig(api_prefix, ["search", "notused1"]), **default_service_settings),
        Service(config, "mail", PortConfig(expose, "3007"), APIConfig(api_prefix, ["mail", "feedback", "code"]), **default_service_settings),
        Service(config, "notification", PortConfig(expose, "3008"), APIConfig(api_prefix, ["send_notification", "read_notification", "unread_notifications"]), ["binaries"], [], [proxy_network, "database"], proxy_network),
        Service(config, "telemetry", PortConfig(expose, "3009"), APIConfig(api_prefix, ["telemetry", "notused1"]), **default_service_settings),
        Service(config, "chat", PortConfig(expose, "3012"), APIConfig(api_prefix, ["chat", "notused1"]), **default_service_settings),
        Service(config, "renderer", PortConfig(expose, "3015"), APIConfig(api_prefix, ["generate-report", "notused1"]), [], [], [proxy_network, "report"], proxy_network),
        Service(config, "report", PortConfig(expose, "3011"), APIConfig(api_prefix, ["report", "notused2"]), ["binaries", "renderer"], [("binaries", "/data/binaries")], [proxy_network, "report"], proxy_network),
        Service(config, "cloc", PortConfig(expose, "3013"), APIConfig(api_prefix, ["cloc", "notused1"]), ["binaries"], [("binaries", "/data/binaries"), ("repo", "/repositories")], [proxy_network, "database"], proxy_network),
        Service(config, "event", PortConfig(expose, "3010"), APIConfig(api_prefix, ["notification", "event"]), **default_service_settings),
        Service(config, "database", PortConfig(open_database, "27017"), None, [], [("database", "/data/db"), ("backup", "/mongo_backup")], [proxy_network, "database"], proxy_network)
    ]


class Service:
    service_name: str

    config: dict

    port_config: Optional[PortConfig]

    api_config: Optional[APIConfig]

    proxy_network: str

    depends_on: List[str]
    volumes: List[tuple[str, str]]
    networks: List[str]
    


    def __init__(self, config, service_name: str, 
                 port_config: Optional[PortConfig], api_config: Optional[APIConfig], 
                 depends_on: List[str], 
                 volumes: List[tuple[str, str]], networks: List[str], proxy_network: str) -> None:
        self.config = config
        self.service_name = service_name
        self.port_config = port_config
        self.api_config = api_config
        self.depends_on = depends_on
        self.volumes = volumes
        self.networks = networks
        self.proxy_network = proxy_network
    
    def __str__(self) -> str:
        folder = self.service_name if self.service_name != "binaries" else ""
        folder = folder if folder != "database" else "mongo"

        depend_on_template = "\n    depends_on:\n" if len(self.depends_on) > 0 else ""
        for container_name in self.depends_on:
            depend_on_template += f"      - {self.config['container_namespace']}-{container_name}\n"
        depend_on_template = depend_on_template[:-1]
            
        volumes_template = "volumes:\n" if len(self.volumes) > 0 else ""
        for (volume_name, volume_path) in self.volumes:
            volumes_template += f"      - {self.config['volume_namespace']}-{volume_name}:{volume_path}\n"
        volumes_template = volumes_template[:-1]
        
        networks_template = "networks:\n" if len(self.networks) > 0 else ""
        for network in self.networks:
            if network != self.proxy_network:
                networks_template += f"      - {self.config['network_namespace']}-{network}\n"
            else:
                networks_template += f"      - {network}\n"
        networks_template = networks_template[:-1]
      
       

        port_template = ""
        optional_duplicate = ":" + self.port_config.port if self.port_config.expose == "ports" else ""
        port_template = f"    {self.port_config.expose}:\n      - {self.port_config.port}{optional_duplicate}\n"
        
        virtual_path_template = ""
        if self.api_config is not None:
            post_prefixes = "("
            for (i, post_prefix) in enumerate(self.api_config.post_prefixes):
                if i > 0:
                    post_prefixes += f"|{post_prefix}"
                    continue
                post_prefixes += post_prefix
            post_prefixes += ")"
            virtual_path_template = f"\n      VIRTUAL_PATH: ~^/{self.api_config.api_prefix}/{post_prefixes}"

        return f"""  {self.config['container_namespace']}-{self.service_name}:{depend_on_template}
    build: ./{folder}
{port_template}    {volumes_template}
    environment:
      VIRTUAL_HOST: "${{VIRTUAL_HOST}}"{virtual_path_template}
      <<: *common-variables
    {networks_template}
"""
        

def create_docker_compose(config):
    print(config)
    services_str = "\n".join([str(service) for service in get_services(config, config['api_prefix'], config['proxy_network'], "expose", config['open_database'])])

    return f"""
x-common-variables: &common-variables
  MONGOURI: "mongodb://${{MONGO_LOGIN}}:${{MONGO_PASSWORD}}@{config['container_namespace']}-database:27017/?authSource=admin"
  JWT_SECRET: "${{JWT_SECRET}}"
  HELLO_MAIL_ADDRESS: "${{HELLO_MAIL_ADDRESS}}"
  HELLO_MAIL_PASSWORD: "${{HELLO_MAIL_PASSWORD}}"
  ADMIN_CREATION_PASSWORD: "${{ADMIN_CREATION_PASSWORD}}"
  AUDITORS_SERVICE_URL: "${{AUDITORS_SERVICE_URL}}"
  AUDITS_SERVICE_URL: "${{AUDITS_SERVICE_URL}}"
  CUSTOMERS_SERVICE_URL: "${{CUSTOMERS_SERVICE_URL}}"
  FILES_SERVICE_URL: "${{FILES_SERVICE_URL}}"
  MAIL_SERVICE_URL: "${{MAIL_SERVICE_URL}}"
  SEARCH_SERVICE_URL: "${{SEARCH_SERVICE_URL}}"
  USERS_SERVICE_URL: "${{USERS_SERVICE_URL}}"
  RENDERER_SERVICE_URL: "${{RENDERER_SERVICE_URL}}"
  NOTIFICATIONS_SERVICE_URL: "${{NOTIFICATIONS_SERVICE_URL}}"
  EVENTS_SERVICE_URL: "${{EVENTS_SERVICE_URL}}"
  API_PREFIX: "${{API_PREFIX}}"
  FRONTEND: "${{FRONTEND}}"
  PROTOCOL: "${{PROTOCOL}}"
  FEEDBACK_EMAIL: "${{FEEDBACK_EMAIL}}"
  GITHUB_CLIENT_SECRET: "${{GITHUB_CLIENT_SECRET}}"
  GITHUB_CLIENT_ID: "${{GITHUB_CLIENT_ID}}"
  RUST_LOG: actix=info,reqwest=info,search=info,common=info,audits=trace
  TIMEOUT: "60"

services:
{services_str}


volumes:
  {config['volume_namespace']}-backup:
  {config['volume_namespace']}-database:
  {config['volume_namespace']}-files:
  {config['volume_namespace']}-binaries:
  {config['volume_namespace']}-repo:
networks:
  {config['network_namespace']}-report:
  {config['network_namespace']}-database:
  {config['proxy_network']}:"""


def create_docker_build(is_test_server):
    features = "--features \"test_server\"" if is_test_server else ""

    return f"""
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
RUN cargo build --release {features}
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
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
        "api_prefix": "api",
        "proxy_network": "nginx-proxy"
    },

    "prod": {
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "prod",
        "volume_namespace": "prod",
        "network_namespace": "prod",
        "proxy_address": "auditdb.io",
        "api_prefix": "api",
        "proxy_network": "nginx-proxy"

    },
    "test": {
        "open_database": True,
        "with_proxy": True,
        "container_namespace": "test",
        "volume_namespace": "test",
        "network_namespace": "test",
        "proxy_address": "dev.auditdb.io",
        "features": '"test_server"',
        "api_prefix": "api",
        "proxy_network": "nginx-proxy"


    },
    "preprod": {
        "open_database": False,
        "with_proxy": True,
        "container_namespace": "preprod",
        "volume_namespace": "preprod",
        "network_namespace": "preprod",
        "proxy_address": "preprod.auditdb.io",
        "api_prefix": "api",
        "proxy_network": "nginx-proxy"
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

    compose = create_docker_compose(config)
    docker = create_docker_build(config)

    
    with open("docker-compose.yml", "w") as f:
        f.write(compose)

    with open("Dockerfile", "w") as f:
        f.write(docker)

if __name__ == "__main__":
    main()