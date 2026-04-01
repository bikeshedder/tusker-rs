DROP INDEX "public"."employees_tenant_id_idx";

DROP INDEX "public"."employees_email_uidx";

ALTER TABLE "public"."employees" DROP CONSTRAINT "employees_pkey";

DROP TABLE "public"."employees";
