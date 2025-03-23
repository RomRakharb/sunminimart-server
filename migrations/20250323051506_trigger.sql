-- Add migration script here
USE sunminimart;

CREATE TRIGGER delete_row_when_amount_zero
AFTER UPDATE ON stocks
FOR EACH ROW
BEGIN
    -- If the Amount is 0 after the update, delete the row
    IF NEW.Amount = 0 THEN
        DELETE FROM stocks WHERE ID = OLD.ID;
    END IF;
END;

