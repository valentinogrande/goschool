use sqlx::MySqlPool;
use crate::structs::{MySelf, PubUser, Role};

impl MySelf {
    /// Check if the current user can chat with a target user based on roles and relationships
    pub async fn can_chat_with(&self, pool: &MySqlPool, target_user_id: u64) -> Result<bool, sqlx::Error> {
        // Self-chat not allowed
        if self.id == target_user_id {
            return Ok(false);
        }

        match self.role {
            Role::admin => {
                // Admins can chat with anyone
                Ok(true)
            }

            Role::student => {
                // Students can chat with:
                // 1. Other students in the same course
                // 2. Teachers/preceptors/admins in their course or who teach their subjects
                let query = r#"
                    SELECT EXISTS(
                        -- Same course students
                        SELECT 1 FROM users u1
                        JOIN users u2 ON u1.course_id = u2.course_id
                        WHERE u1.id = ? AND u2.id = ? AND u2.id != u1.id

                        UNION

                        -- Teachers of their subjects
                        SELECT 1 FROM users student
                        JOIN subjects s ON s.course_id = student.course_id
                        WHERE student.id = ? AND s.teacher_id = ?

                        UNION

                        -- Preceptor of their course
                        SELECT 1 FROM users student
                        JOIN courses c ON c.id = student.course_id
                        WHERE student.id = ? AND c.preceptor_id = ?

                        UNION

                        -- Any admin
                        SELECT 1 FROM users u
                        JOIN roles r ON r.user_id = u.id
                        WHERE u.id = ? AND r.role = 'admin'
                    ) as can_chat
                "#;

                sqlx::query_scalar(query)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .fetch_one(pool)
                    .await
            }

            Role::teacher => {
                // Teachers can chat with:
                // 1. Students in subjects they teach
                // 2. Other teachers
                // 3. Preceptors of courses they teach
                // 4. Admins
                let query = r#"
                    SELECT EXISTS(
                        -- Students in their subjects
                        SELECT 1 FROM subjects s
                        JOIN users student ON student.course_id = s.course_id
                        WHERE s.teacher_id = ? AND student.id = ?

                        UNION

                        -- Other teachers
                        SELECT 1 FROM roles r
                        WHERE r.user_id = ? AND r.role = 'teacher'

                        UNION

                        -- Preceptors of courses they teach
                        SELECT 1 FROM subjects s
                        JOIN courses c ON c.id = s.course_id
                        WHERE s.teacher_id = ? AND c.preceptor_id = ?

                        UNION

                        -- Admins
                        SELECT 1 FROM roles r
                        WHERE r.user_id = ? AND r.role = 'admin'
                    ) as can_chat
                "#;

                sqlx::query_scalar(query)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .fetch_one(pool)
                    .await
            }

            Role::preceptor => {
                // Preceptors can chat with:
                // 1. Students in their course
                // 2. Teachers who teach subjects in their course
                // 3. Other preceptors
                // 4. Admins
                let query = r#"
                    SELECT EXISTS(
                        -- Students in their course
                        SELECT 1 FROM courses c
                        JOIN users student ON student.course_id = c.id
                        WHERE c.preceptor_id = ? AND student.id = ?

                        UNION

                        -- Teachers in their course
                        SELECT 1 FROM courses c
                        JOIN subjects s ON s.course_id = c.id
                        WHERE c.preceptor_id = ? AND s.teacher_id = ?

                        UNION

                        -- Other preceptors
                        SELECT 1 FROM roles r
                        WHERE r.user_id = ? AND r.role = 'preceptor'

                        UNION

                        -- Admins
                        SELECT 1 FROM roles r
                        WHERE r.user_id = ? AND r.role = 'admin'
                    ) as can_chat
                "#;

                sqlx::query_scalar(query)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .fetch_one(pool)
                    .await
            }

            Role::father => {
                // Fathers can chat with:
                // 1. Teachers/preceptors of their children's courses
                // 2. Admins
                let query = r#"
                    SELECT EXISTS(
                        -- Teachers of children's subjects
                        SELECT 1 FROM families f
                        JOIN users student ON student.id = f.student_id
                        JOIN subjects s ON s.course_id = student.course_id
                        WHERE f.father_id = ? AND s.teacher_id = ?

                        UNION

                        -- Preceptors of children's courses
                        SELECT 1 FROM families f
                        JOIN users student ON student.id = f.student_id
                        JOIN courses c ON c.id = student.course_id
                        WHERE f.father_id = ? AND c.preceptor_id = ?

                        UNION

                        -- Admins
                        SELECT 1 FROM roles r
                        WHERE r.user_id = ? AND r.role = 'admin'
                    ) as can_chat
                "#;

                sqlx::query_scalar(query)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(self.id)
                    .bind(target_user_id)
                    .bind(target_user_id)
                    .fetch_one(pool)
                    .await
            }
        }
    }

    /// Get list of users that the current user can chat with
    pub async fn get_available_chat_users(&self, pool: &MySqlPool) -> Result<Vec<PubUser>, sqlx::Error> {
        match self.role {
            Role::admin => {
                // Admins can chat with everyone
                let query = r#"
                    SELECT DISTINCT u.id, u.email, u.photo, u.course_id, pd.full_name
                    FROM users u
                    LEFT JOIN personal_data pd ON u.id = pd.user_id
                    WHERE u.id != ?
                    ORDER BY pd.full_name, u.email
                "#;

                sqlx::query_as::<_, PubUser>(query)
                    .bind(self.id)
                    .fetch_all(pool)
                    .await
            }

            Role::student => {
                // Students can chat with:
                // 1. Peers in same course
                // 2. Teachers of their subjects
                // 3. Their preceptor
                // 4. Admins
                let query = r#"
                    SELECT DISTINCT u.id, u.email, u.photo, u.course_id, pd.full_name
                    FROM users u
                    LEFT JOIN personal_data pd ON u.id = pd.user_id
                    WHERE u.id != ? AND (
                        -- Same course peers
                        u.course_id = (SELECT course_id FROM users WHERE id = ?)

                        OR

                        -- Teachers of their subjects
                        u.id IN (
                            SELECT s.teacher_id FROM subjects s
                            WHERE s.course_id = (SELECT course_id FROM users WHERE id = ?)
                        )

                        OR

                        -- Their preceptor
                        u.id = (
                            SELECT c.preceptor_id FROM courses c
                            WHERE c.id = (SELECT course_id FROM users WHERE id = ?)
                        )

                        OR

                        -- Admins
                        u.id IN (SELECT user_id FROM roles WHERE role = 'admin')
                    )
                    ORDER BY pd.full_name, u.email
                "#;

                sqlx::query_as::<_, PubUser>(query)
                    .bind(self.id)
                    .bind(self.id)
                    .bind(self.id)
                    .bind(self.id)
                    .fetch_all(pool)
                    .await
            }

            Role::teacher => {
                // Teachers can chat with:
                // 1. Students in their subjects
                // 2. Other teachers
                // 3. Preceptors
                // 4. Admins
                let query = r#"
                    SELECT DISTINCT u.id, u.email, u.photo, u.course_id, pd.full_name
                    FROM users u
                    LEFT JOIN personal_data pd ON u.id = pd.user_id
                    WHERE u.id != ? AND (
                        -- Students in their subjects
                        u.course_id IN (
                            SELECT s.course_id FROM subjects s WHERE s.teacher_id = ?
                        )

                        OR

                        -- Other teachers
                        u.id IN (SELECT user_id FROM roles WHERE role = 'teacher')

                        OR

                        -- Preceptors
                        u.id IN (SELECT user_id FROM roles WHERE role = 'preceptor')

                        OR

                        -- Admins
                        u.id IN (SELECT user_id FROM roles WHERE role = 'admin')
                    )
                    ORDER BY pd.full_name, u.email
                "#;

                sqlx::query_as::<_, PubUser>(query)
                    .bind(self.id)
                    .bind(self.id)
                    .fetch_all(pool)
                    .await
            }

            Role::preceptor => {
                // Preceptors can chat with:
                // 1. Students in their course
                // 2. Teachers in their course
                // 3. Other preceptors
                // 4. Admins
                let query = r#"
                    SELECT DISTINCT u.id, u.email, u.photo, u.course_id, pd.full_name
                    FROM users u
                    LEFT JOIN personal_data pd ON u.id = pd.user_id
                    WHERE u.id != ? AND (
                        -- Students in their course
                        u.course_id IN (
                            SELECT c.id FROM courses c WHERE c.preceptor_id = ?
                        )

                        OR

                        -- Teachers in their course
                        u.id IN (
                            SELECT s.teacher_id FROM subjects s
                            WHERE s.course_id IN (
                                SELECT c.id FROM courses c WHERE c.preceptor_id = ?
                            )
                        )

                        OR

                        -- Other preceptors
                        u.id IN (SELECT user_id FROM roles WHERE role = 'preceptor')

                        OR

                        -- Admins
                        u.id IN (SELECT user_id FROM roles WHERE role = 'admin')
                    )
                    ORDER BY pd.full_name, u.email
                "#;

                sqlx::query_as::<_, PubUser>(query)
                    .bind(self.id)
                    .bind(self.id)
                    .bind(self.id)
                    .fetch_all(pool)
                    .await
            }

            Role::father => {
                // Fathers can chat with:
                // 1. Teachers of their children's subjects
                // 2. Preceptors of their children's courses
                // 3. Admins
                let query = r#"
                    SELECT DISTINCT u.id, u.email, u.photo, u.course_id, pd.full_name
                    FROM users u
                    LEFT JOIN personal_data pd ON u.id = pd.user_id
                    WHERE u.id != ? AND (
                        -- Teachers of children's subjects
                        u.id IN (
                            SELECT s.teacher_id FROM subjects s
                            WHERE s.course_id IN (
                                SELECT student.course_id FROM users student
                                JOIN families f ON f.student_id = student.id
                                WHERE f.father_id = ?
                            )
                        )

                        OR

                        -- Preceptors of children's courses
                        u.id IN (
                            SELECT c.preceptor_id FROM courses c
                            WHERE c.id IN (
                                SELECT student.course_id FROM users student
                                JOIN families f ON f.student_id = student.id
                                WHERE f.father_id = ?
                            )
                        )

                        OR

                        -- Admins
                        u.id IN (SELECT user_id FROM roles WHERE role = 'admin')
                    )
                    ORDER BY pd.full_name, u.email
                "#;

                sqlx::query_as::<_, PubUser>(query)
                    .bind(self.id)
                    .bind(self.id)
                    .bind(self.id)
                    .fetch_all(pool)
                    .await
            }
        }
    }

    /// Check if user is a participant of a specific chat
    pub async fn is_chat_participant(&self, pool: &MySqlPool, chat_id: u64) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)"
        )
        .bind(self.id)
        .bind(chat_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    /// Check if user is an admin of a group chat
    pub async fn is_chat_admin(&self, pool: &MySqlPool, chat_id: u64) -> Result<bool, sqlx::Error> {
        let is_admin: bool = sqlx::query_scalar(
            "SELECT is_admin FROM chat_participants WHERE user_id = ? AND chat_id = ?"
        )
        .bind(self.id)
        .bind(chat_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false);

        Ok(is_admin)
    }
}
