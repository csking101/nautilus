import sys
import json
from sklearn.datasets import load_iris
from sklearn.model_selection import train_test_split
from sklearn.linear_model import LogisticRegression

def main():
    # Accept data path argument (not used in dummy example)
    data_path = sys.argv[1] if len(sys.argv) > 1 else None

    # Load dummy dataset
    X, y = load_iris(return_X_y=True)
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

    # Train simple model
    clf = LogisticRegression(max_iter=100)
    clf.fit(X_train, y_train)
    accuracy = clf.score(X_test, y_test)
    loss = 1 - accuracy

    # Output result as JSON
    print(json.dumps({"accuracy": int(accuracy), "loss": int(loss)}))

if __name__ == "__main__":
    main()
