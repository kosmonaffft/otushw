import json
import random
import urllib.request
import urllib.parse
import urllib.error
import csv
import os

if __name__ == "__main__":
    ids_file = open('data/ids_100002.csv', 'r', encoding='utf-8')
    number = 0
    for id_row in ids_file:
        url = "http://localhost:8080/login"
        data = {
            "id": id_row.strip(),
            "password": "1234567890"
        }
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "PythonScript/1.0"
        }
        json_data = json.dumps(data).encode('utf-8')
        req = urllib.request.Request(url, data=json_data, headers=headers, method='POST')
        token = None
        with urllib.request.urlopen(req) as response:
            response_data = json.loads(response.read().decode('utf-8'))
            token = response_data

        big_russian_letters = 'ЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ'
        random_chars1 = ''.join(random.choice(big_russian_letters) for _ in range(1))
        random_chars2 = ''.join(random.choice(big_russian_letters) for _ in range(1))
        search_url = "http://localhost:8080/search"  # замените на реальный URL
        params = {'f': random_chars1, 's': random_chars2}
        url = f"{search_url}?{urllib.parse.urlencode(params)}"
        headers = {
            "Authorization": f"Bearer {token}",
            "User-Agent": "PythonScript/1.0"
        }
        req = urllib.request.Request(url, headers=headers, method='GET')
        friends = []
        with urllib.request.urlopen(req) as response:
            response_data = json.loads(response.read().decode('utf-8'))
            friends = response_data

        for friend in friends:
            id = friend['id']
            url = f"http://localhost:8080/friends/{id}"
            headers = {
                "Authorization": f"Bearer {token}",
                "User-Agent": "PythonScript/1.0"
            }
            req = urllib.request.Request(url, headers=headers, method='POST')
            with urllib.request.urlopen(req) as response:
                response_data = response.read().decode('utf-8')
                pass
        number = number + 1
        print(f"Processed id number {number}")

    ids_file.close()
