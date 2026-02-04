import init, { BlogApp } from './pkg/blog_wasm.js';

let app = null;
let currentUserId = null;
let currentPage = 0;
const POSTS_PER_PAGE = 5;

// Декодирование JWT токена для извлечения user_id
function decodeJWT(token) {
    try {
        const parts = token.split('.');
        if (parts.length !== 3) {
            return null;
        }

        const payload = parts[1];

        const base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(
            atob(base64)
                .split('')
                .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
                .join('')
        );

        return JSON.parse(jsonPayload);
    } catch (error) {
        console.error('Ошибка декодирования JWT:', error);
        return null;
    }
}

// Получение ID текущего пользователя из токена
function getCurrentUserId() {
    if (!app || !app.is_authenticated()) {
        return null;
    }

    const token = app.get_token();
    if (!token) {
        return null;
    }

    const payload = decodeJWT(token);
    if (!payload) {
        return null;
    }

    return payload.user_id;
}

// Инициализация приложения
async function initApp() {
    try {
        await init();

        const serverUrl = localStorage.getItem('blog_server') || 'http://127.0.0.1:3000';
        app = new BlogApp(serverUrl);

        try {
            await app.get_token_from_storage();
            currentUserId = getCurrentUserId();
            updateAuthUI();
        } catch (e) {
            console.log('Токен не найден в localStorage');
        }

        await loadPosts();
        setupEventListeners();
    } catch (error) {
        console.error('Ошибка инициализации:', error);
        showError('posts-error', 'Ошибка инициализации приложения');
    }
}

// Настройка обработчиков событий
function setupEventListeners() {
    // Переключение между формами входа и регистрации
    document.getElementById('show-login-tab').addEventListener('click', () => {
        showLoginForm();
    });

    document.getElementById('show-register-tab').addEventListener('click', () => {
        showRegisterForm();
    });

    // Форма входа
    document.getElementById('login-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        await handleLogin();
    });

    document.getElementById('logout-btn').addEventListener('click', async (e) => {
        e.preventDefault();
        await handleLogout();
    })

    // Форма регистрации
    document.getElementById('register-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        await handleRegister();
    });

    // Форма создания поста
    document.getElementById('create-post-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        await handleCreatePost();
    });

    // Форма редактирования поста
    document.getElementById('edit-post-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        await handleUpdatePost();
    });

    // Кнопка отмены редактирования
    document.getElementById('cancel-edit-btn').addEventListener('click', () => {
        document.getElementById('edit-post-panel').style.display = 'none';
        document.getElementById('create-post-panel').style.display = app.is_authenticated() ? 'block' : 'none';
    });

    // Кнопка выхода
    document.getElementById('prev-page-btn').addEventListener('click', async () => {
        await goToPreviousPage();
    });

    document.getElementById('next-page-btn').addEventListener('click', async () => {
        await goToNextPage();
    });
}

// Обработка входа
async function handleLogin() {
    const username = document.getElementById('login-username').value.trim();
    const password = document.getElementById('login-password').value.trim();

    if (!username || !password) {
        showError('login-error', 'Заполните все поля');
        return;
    }

    try {
        await app.login(username, password);
        await app.save_token_to_storage();

        currentUserId = getCurrentUserId();

        document.getElementById('login-form').reset();
        clearError('login-error');

        updateAuthUI();
        currentPage = 0;
        await loadPosts();
    } catch (error) {
        showError('login-error', `Ошибка входа: ${error}`);
    }
}

// Обработка регистрации
async function handleRegister() {
    const username = document.getElementById('register-username').value.trim();
    const email = document.getElementById('register-email').value.trim();
    const password = document.getElementById('register-password').value.trim();

    if (!username || !email || !password) {
        showError('register-error', 'Заполните все поля');
        return;
    }

    try {
        await app.register(username, email, password);
        await app.save_token_to_storage();

        currentUserId = getCurrentUserId();

        document.getElementById('register-form').reset();
        clearError('register-error');

        updateAuthUI();
        currentPage = 0;
        await loadPosts();
    } catch (error) {
        showError('register-error', `Ошибка регистрации: ${error}`);
    }
}

// Обработка создания поста
async function handleCreatePost() {
    const title = document.getElementById('post-title').value.trim();
    const content = document.getElementById('post-content').value.trim();

    if (!title || !content) {
        showError('create-post-error', 'Заполните все поля');
        return;
    }

    try {
        await app.create_post(title, content);

        document.getElementById('create-post-form').reset();
        clearError('create-post-error');

        currentPage = 0;
        await loadPosts();
    } catch (error) {
        showError('create-post-error', `Ошибка создания поста: ${error}`);
    }
}

// Обработка обновления поста
async function handleUpdatePost() {
    const id = BigInt(parseInt(document.getElementById('edit-post-id').value));
    const title = document.getElementById('edit-post-title').value.trim();
    const content = document.getElementById('edit-post-content').value.trim();

    if (!title || !content) {
        showError('edit-post-error', 'Заполните все поля');
        return;
    }

    try {
        await app.update_post(id, title, content);

        document.getElementById('edit-post-panel').style.display = 'none';
        document.getElementById('create-post-panel').style.display = 'block';
        clearError('edit-post-error');

        await loadPosts();
    } catch (error) {
        showError('edit-post-error', `Ошибка обновления поста: ${error}`);
    }
}

// Обработка удаления поста
async function handleDeletePost(postId) {
    if (!confirm('Вы уверены, что хотите удалить этот пост?')) {
        return;
    }

    try {
        await app.delete_post(BigInt(postId));
        await loadPosts();
    } catch (error) {
        showError('posts-error', `Ошибка удаления поста: ${error}`);
    }
}

// Загрузка постов с пагинацией
async function loadPosts() {
    const postsLoading = document.getElementById('posts-loading');
    const postsList = document.getElementById('posts-list');

    postsLoading.style.display = 'block';
    postsList.innerHTML = '';
    clearError('posts-error');

    try {
        const offset = currentPage * POSTS_PER_PAGE;
        const posts = await app.load_posts(BigInt(POSTS_PER_PAGE + 1), BigInt(offset));
        postsLoading.style.display = 'none';

        if (posts.length === 0 && currentPage === 0) {
            postsList.innerHTML = '<p class="loading">Постов пока что нет</p>';
            updatePaginationButtons(false, false);
            return;
        }

        if (posts.length === 0 && currentPage > 0) {
            currentPage--;
            await loadPosts();
            return;
        }

        posts.forEach(post => {
            if (postsList.childElementCount === POSTS_PER_PAGE) {
                return;
            }

            const postCard = createPostCard(post);
            postsList.appendChild(postCard);
        });

        updatePaginationButtons(currentPage > 0, posts.length === POSTS_PER_PAGE + 1);
    } catch (error) {
        postsLoading.style.display = 'none';
        showError('posts-error', `Ошибка загрузки постов: ${error}`);
    }
}

// Создание карточки поста
function createPostCard(post) {
    const card = document.createElement('div');
    card.className = 'post-card';

    const isAuthor = currentUserId !== null && currentUserId === post.author_id;

    card.innerHTML = `
        <div class="post-header">
            <div>
                <h3 class="post-title">${escapeHtml(post.title)}</h3>
                <div class="post-meta">
                    Идентификатор автора: ${post.author_id} |
                    Создан: ${formatDate(post.created_at)} |
                    Обновлен: ${formatDate(post.updated_at)}
                </div>
            </div>
        </div>
        <div class="post-content">${escapeHtml(post.content)}</div>
        ${isAuthor ? `
        <div class="post-actions">
            <button class="edit-btn" onclick="window.editPost(${post.id}, '${escapeHtml(post.title)}', '${escapeHtml(post.content)}')">
                Редактировать
            </button>
            <button class="delete-btn" onclick="window.deletePost(${post.id})">
                Удалить
            </button>
        </div>
        ` : ''}
    `;

    return card;
}

// Редактирование поста
window.editPost = function(id, title, content) {
    document.getElementById('edit-post-id').value = id;
    document.getElementById('edit-post-title').value = title;
    document.getElementById('edit-post-content').value = content;

    document.getElementById('create-post-panel').style.display = 'none';
    document.getElementById('edit-post-panel').style.display = 'block';

    document.getElementById('edit-post-panel').scrollIntoView({ behavior: 'smooth' });
};

// Удаление поста
window.deletePost = async function(id) {
    await handleDeletePost(id);
};

// Обработка выхода
async function handleLogout() {
    try {
        await app.remove_token_from_storage();
        currentUserId = null;
        updateAuthUI();
        currentPage = 0;
        await loadPosts();
    } catch (error) {
        console.error('Ошибка выхода:', error);
    }
}

// Обновление UI в зависимости от статуса аутентификации
function updateAuthUI() {
    const isAuthenticated = app.is_authenticated();

    document.getElementById('status-text').textContent = isAuthenticated ? 'Авторизован' : 'Не авторизован';
    document.getElementById('logout-btn').style.display = isAuthenticated ? 'block' : 'none';
    document.getElementById('auth-panel').style.display = isAuthenticated ? 'none' : 'block';
    document.getElementById('create-post-panel').style.display = isAuthenticated ? 'block' : 'none';
}

// Отобразить ошибку
function showError(elementId, message) {
    const element = document.getElementById(elementId);
    element.textContent = message;
    element.style.display = 'block';
}

function clearError(elementId) {
    const element = document.getElementById(elementId);
    element.textContent = '';
    element.style.display = 'none';
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDate(dateString) {
    try {
        const date = new Date(dateString);
        return date.toLocaleString('ru-RU');
    } catch (e) {
        return dateString;
    }
}

// Переключение на форму входа
function showLoginForm() {
    document.getElementById('login-container').style.display = 'block';
    document.getElementById('register-container').style.display = 'none';
    document.getElementById('show-login-tab').classList.add('active');
    document.getElementById('show-register-tab').classList.remove('active');
    clearError('login-error');
    clearError('register-error');
}

// Переключение на форму регистрации
function showRegisterForm() {
    document.getElementById('login-container').style.display = 'none';
    document.getElementById('register-container').style.display = 'block';
    document.getElementById('show-login-tab').classList.remove('active');
    document.getElementById('show-register-tab').classList.add('active');
    clearError('login-error');
    clearError('register-error');
}

// Навигация по страницам (назад)
async function goToPreviousPage() {
    if (currentPage > 0) {
        currentPage--;
        await loadPosts();
    }
}

// Навигация по страницам (вперед)
async function goToNextPage() {
    currentPage++;
    await loadPosts();
}

// Обновление состояния кнопок пагинации
function updatePaginationButtons(hasPrevious, hasNext) {
    const prevBtn = document.getElementById('prev-page-btn');
    const nextBtn = document.getElementById('next-page-btn');
    const pageInfo = document.getElementById('page-info');

    if (prevBtn) {
        prevBtn.disabled = !hasPrevious;
        prevBtn.style.opacity = hasPrevious ? '1' : '0.5';
    }

    if (nextBtn) {
        nextBtn.disabled = !hasNext;
        nextBtn.style.opacity = hasNext ? '1' : '0.5';
    }

    if (pageInfo) {
        pageInfo.textContent = `Страница ${currentPage + 1}`;
    }
}

await initApp();
