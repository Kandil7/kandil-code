from fastapi import FastAPI
from pydantic import BaseModel
import time

app = FastAPI(title="{{project_name}}")

class Item(BaseModel):
    name: str
    description: str = None

@app.get("/")
def read_root():
    return {"Hello": "World"}

@app.get("/items/{item_id}")
def read_item(item_id: int, q: str = None):
    return {"item_id": item_id, "q": q}

@app.post("/items/")
def create_item(item: Item):
    return item

@app.get("/healthz")
def healthz():
    return {"status": "ok"}

@app.get("/readyz")
def readyz():
    start = time.time()
    elapsed_ms = int((time.time() - start) * 1000)
    return {"ready": True, "elapsed_ms": elapsed_ms}
