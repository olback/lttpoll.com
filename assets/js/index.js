window.addEventListener('DOMContentLoaded', () => {
    document.getElementById("row-add").addEventListener('click', () => {
        let el = document.getElementById("answers");
        if (el.children.length < 10) {
            let ansRow = document.createElement('div');
            ansRow.className = "answer-row";
            let ansLabel = document.createElement('label');
            ansLabel.textContent = `Answer ${el.children.length + 1}`;
            ansLabel.htmlFor = `answer-${el.children.length + 1}`;
            let ansInput = document.createElement('input');
            ansInput.id = `answer-${el.children.length + 1}`;
            ansInput.name = 'answer[]';
            ansInput.type = 'text';
            ansRow.appendChild(ansLabel);
            ansRow.appendChild(ansInput);
            el.appendChild(ansRow);
        }
    });
    document.getElementById("row-del").addEventListener('click', () => {
        let el = document.getElementById("answers");
        if (el.children.length > 2) {
            el.children.item(el.children.length - 1).remove();
        }
    });
});
