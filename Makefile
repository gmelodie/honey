.PHONY: deploy logs

deploy:
	docker compose up -d --build --remove-orphans
	docker image prune -f

logs:
	docker compose logs -f --tail=100
