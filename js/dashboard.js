

        // 封装用户信息获取逻辑
        function fetchUserInfo() {
            fetch('/api/get_username', {
                method: 'GET',
            })
            .then(handleResponse)
            .then(updateUI);
        }
        function handleResponse(response) {
            if (!response.ok) throw new Error(`HTTP 错误: ${response.status}`);
            return response.json();
        }
        function updateUI(data) {
            document.querySelector('.username').textContent = `当前用户：${data.username}`;
        }
        document.addEventListener('DOMContentLoaded', fetchUserInfo);
