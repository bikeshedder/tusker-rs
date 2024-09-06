ALTER TABLE "public"."a" DROP CONSTRAINT "a_pkey";

ALTER TABLE "public"."a" DROP CONSTRAINT "a_age_check";

ALTER TABLE "public"."a" DROP CONSTRAINT "a_name_check";

ALTER TABLE "public"."a"
    ALTER COLUMN "id" bigint,
    ALTER COLUMN "name" character varying(50);
