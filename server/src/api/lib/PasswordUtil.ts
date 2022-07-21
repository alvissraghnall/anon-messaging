import bcrypt from "bcrypt";
// hash password method, as well as compare hashes.
export class PasswordUtil {

    static async hashPassword (password: string, saltRounds = 10) {
        try {
            // Generate a salt
            const salt = await bcrypt.genSalt(saltRounds);
    
            // Hash password
            return await bcrypt.hash(password, salt);
        } catch (error) {
            console.log(error);
        }
        // Return null if error
        return null;
    }

    static async comparePassword (password: string, hash: string) {
        try {
            // Compare password
            return await bcrypt.compare(password, hash);
        } catch (error) {
            console.log(error);
        }
    
        // Return false if error
        return false;
    };
    
}